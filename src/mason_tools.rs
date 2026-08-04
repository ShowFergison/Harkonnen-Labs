//! Mason's opt-in tool-call loop.
//!
//! Every other Mason transport in this codebase is single-shot: one prompt, one
//! reply, and whatever the reply contains is the whole of the model's output.
//! That works when the model already knows everything it needs. It does not
//! work when the model needs to *look* — read a file it was not given, or list
//! a directory to find out what is there — before it can decide what to write.
//!
//! This module adds that missing capability, and nothing else. It is dead
//! unless a spec sets `worker_harness.tool_loop: true`.
//!
//! Two rules shape everything here, both learned the expensive way from the six
//! transports that came before:
//!
//! 1. **Never return `Ok` having silently skipped content.** If the model emits
//!    something this parser does not understand, the parse fails loudly and
//!    names what and where. A rejected response costs a turn; a silently
//!    dropped `write_file` costs the operator a file they believe was written.
//! 2. **Return writes, do not perform them.** The loop hands back
//!    `(path, content)` pairs. The caller converts them to `MasonEdit` and
//!    routes them through `validate_mason_edits` and the one existing apply
//!    path. Two write paths would mean two places for the safety check to be
//!    forgotten, and one of them would eventually be.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::llm::{LlmProvider, LlmRequest, Message};

const TOOL_MARKER: &str = "### TOOL:";
const CONTENT_MARKER: &str = "### CONTENT";
const END_CONTENT_MARKER: &str = "### END CONTENT";

/// Wording handed to the model, kept next to the parser for the same reason
/// `FENCED_FORMAT_INSTRUCTION` is: a prompt describing one format while the
/// parser expects another is the exact failure this transport exists to remove.
pub const TOOL_LOOP_INSTRUCTION: &str = "\
You are working in a tool loop. You may inspect the workspace before deciding \
what to write, one message at a time.

To inspect a file:

### TOOL: read_file
path: <relative/path/from/the/workspace/root>

To list a directory:

### TOOL: list_dir
path: <relative/path/from/the/workspace/root>

To write a file (this is how you make every change — there is no other way):

### TOOL: write_file
path: <relative/path/from/the/workspace/root>
### CONTENT
<the complete contents of the file, written literally>
### END CONTENT

Rules:
- Paths are always relative to the workspace root. Absolute paths and '..' are \
refused outright and end the run.
- read_file and list_dir can reach any file in the workspace, not only the ones \
you may edit. Read what you need to understand the change; do not go looking \
through unrelated files.
- You may emit more than one tool call in a single message; each is executed in \
order and every result is returned to you.
- write_file replaces the whole file. Include the complete contents, exactly as \
they should appear on disk. Do not escape quotes, backslashes or newlines, and \
do not wrap contents in backticks.
- File content must not contain any line whose only non-whitespace text is \
'### END CONTENT', and must not contain a line whose first non-whitespace text \
is '### TOOL:'. Indenting such a line does NOT make it safe: the parser trims \
each line before comparing it, so an indented '### END CONTENT' still ends your \
write and everything after it is lost. If the file you are writing genuinely \
needs one of those lines, say so instead of writing the file.
- Markers are matched exactly: three hashes, uppercase, spelled as shown. \
'#### TOOL:' or '### Tool:' are not tool calls, and a message containing one is \
rejected rather than guessed at.
- Do NOT use '### FILE:' or '### PATCH:' blocks, and do NOT send a JSON edit \
proposal, in this mode. None of them are read here, and a message containing \
one is rejected.
- When you have written everything the spec requires, reply with a short summary \
and no tool calls at all. That ends the loop.";

/// The three things Mason may do inside the staged workspace.
///
/// `WriteFile` is deliberately not an action: it is a *request* to write, which
/// the loop records and returns rather than performing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MasonTool {
    ReadFile { path: String },
    ListDir { path: String },
    WriteFile { path: String, content: String },
}

impl MasonTool {
    pub fn name(&self) -> &'static str {
        match self {
            MasonTool::ReadFile { .. } => "read_file",
            MasonTool::ListDir { .. } => "list_dir",
            MasonTool::WriteFile { .. } => "write_file",
        }
    }

    pub fn path(&self) -> &str {
        match self {
            MasonTool::ReadFile { path }
            | MasonTool::ListDir { path }
            | MasonTool::WriteFile { path, .. } => path,
        }
    }
}

/// Convenience wrapper for the single-call case, and for tests.
///
/// Returns `None` both when the message carries no tool call at all and when it
/// carries something this parser rejects — which makes it unsafe to drive the
/// loop with, because those two cases demand opposite responses ("the model is
/// finished" versus "the model emitted something we could not read"). The loop
/// uses [`parse_tool_calls`], which keeps them apart. This exists because a
/// single call is the overwhelmingly common shape and reads far better in a
/// test than a `Vec` destructure.
pub fn parse_tool_call(raw: &str) -> Option<MasonTool> {
    match parse_tool_calls(raw) {
        Ok(mut calls) if calls.len() == 1 => calls.pop(),
        _ => None,
    }
}

/// Parse every tool call in one model message.
///
/// Every reading of the input this parser does not understand — an unknown tool
/// name, a missing `path:`, a `write_file` with no content block, a content
/// block that never closes, a second `path:` for one call — is an `Err`. None
/// of them are recoverable by guessing, and every guess would drop or invent a
/// file write.
///
/// `Ok(vec![])` means only that no *exact* `### TOOL:` header was found. That is
/// **not** on its own sufficient to conclude the model has finished, and callers
/// must not treat it that way: a message can carry a write in a format this
/// parser does not read at all — a JSON edit proposal, a `### FILE:` block, or
/// the right vocabulary one character off (`#### TOOL:`, `### Tool:`) — and
/// every one of those parses to zero calls here. [`unreadable_write_marker`]
/// exists to catch exactly that, and
/// [`run_mason_tool_loop_with_recorder`] consults it before ever concluding the
/// model is done. Termination in this lane is positive — no calls *and* nothing
/// unreadable — because "I could not read this" and "the model is finished"
/// must never be the same outcome.
pub fn parse_tool_calls(raw: &str) -> Result<Vec<MasonTool>> {
    let mut calls: Vec<MasonTool> = Vec::new();
    let mut pending: Option<Pending> = None;
    let mut in_content = false;

    // Split on '\n' rather than using `lines()` so a trailing '\r' survives
    // into content, matching how `collect_fenced_edits` treats CRLF bodies.
    for (index, line) in raw.split('\n').enumerate() {
        let line_num = index + 1;
        let for_markers = line.trim_end_matches('\r');
        let trimmed = for_markers.trim();

        if in_content {
            if trimmed == END_CONTENT_MARKER {
                in_content = false;
            } else if trimmed.starts_with(TOOL_MARKER) {
                // A new tool header while a content block is open means the
                // model forgot the closer. Everything from here to end of input
                // would otherwise be swallowed into the previous file's
                // content, silently, and the tool call on this line would
                // vanish with it.
                bail!(
                    "line {line_num}: a '{TOOL_MARKER}' header appeared while the \
                     '{CONTENT_MARKER}' block was still open — close it with \
                     '{END_CONTENT_MARKER}' first. File content may not contain a line \
                     starting with '{TOOL_MARKER}'."
                );
            } else if let Some(body) = pending.as_mut().and_then(|p| p.content.as_mut()) {
                body.push(line.to_string());
            } else {
                // Unreachable in practice: `in_content` is only ever set
                // alongside a pending call with an open body. Refusing beats
                // an `unwrap`, and beats dropping the line.
                bail!("line {line_num}: content block has no owning tool call");
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix(TOOL_MARKER) {
            if let Some(previous) = pending.take() {
                calls.push(finish_pending(previous)?);
            }
            let kind = rest.trim().to_string();
            if kind.is_empty() {
                bail!("line {line_num}: '{TOOL_MARKER}' header named no tool");
            }
            pending = Some(Pending {
                kind,
                line: line_num,
                path: None,
                content: None,
            });
        } else if let Some(rest) = trimmed.strip_prefix("path:") {
            let Some(current) = pending.as_mut() else {
                // A bare `path:` line in prose, with no tool call open, is not
                // a tool call and is not treated as one. It is also not
                // silently dropped content: with no header there is nothing to
                // dispatch, and a message with no headers at all is terminal
                // by definition.
                continue;
            };
            if current.path.is_some() {
                bail!(
                    "line {line_num}: '{}' tool call declared a second path — one call, one path",
                    current.kind
                );
            }
            let path = rest.trim().to_string();
            if path.is_empty() {
                bail!(
                    "line {line_num}: '{}' tool call declared an empty path",
                    current.kind
                );
            }
            if path.len() > 200 {
                bail!(
                    "line {line_num}: '{}' tool call declared an implausible {} character path",
                    current.kind,
                    path.len()
                );
            }
            current.path = Some(path);
        } else if trimmed == CONTENT_MARKER {
            let Some(current) = pending.as_mut() else {
                bail!(
                    "line {line_num}: '{CONTENT_MARKER}' appeared outside any tool call — a \
                     content block belongs to a '{TOOL_MARKER} write_file' header."
                );
            };
            if current.content.is_some() {
                bail!(
                    "line {line_num}: '{}' tool call opened a second '{CONTENT_MARKER}' block",
                    current.kind
                );
            }
            current.content = Some(Vec::new());
            in_content = true;
        } else if trimmed == END_CONTENT_MARKER {
            bail!(
                "line {line_num}: stray '{END_CONTENT_MARKER}' with no open content block — file \
                 content may not contain a line consisting solely of '{END_CONTENT_MARKER}'."
            );
        }
        // Anything else at top level is prose. The model is allowed to think
        // out loud around its tool calls.
    }

    // An unterminated content block swallowed every line after it. Returning
    // the calls collected before it would hand back a truncated file as if it
    // were whole — the single worst outcome this module can produce.
    if in_content {
        let path = pending
            .as_ref()
            .and_then(|p| p.path.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        bail!(
            "the '{CONTENT_MARKER}' block for {path:?} was never closed with \
             '{END_CONTENT_MARKER}' — the response was cut off, so the file content is \
             incomplete. Raise the token budget or write a smaller file."
        );
    }

    if let Some(previous) = pending.take() {
        calls.push(finish_pending(previous)?);
    }

    Ok(calls)
}

/// A call being accumulated by [`parse_tool_calls`]. `content` is `None` until
/// a `### CONTENT` marker opens one, which is what separates "write_file with
/// no content block" (an error) from "write_file whose content is empty" (a
/// legal request to create an empty file).
struct Pending {
    kind: String,
    line: usize,
    path: Option<String>,
    content: Option<Vec<String>>,
}

fn finish_pending(pending: Pending) -> Result<MasonTool> {
    let Pending {
        kind,
        line,
        path,
        content,
    } = pending;
    let Some(path) = path else {
        bail!("line {line}: '{kind}' tool call is missing its 'path:' line");
    };
    match kind.as_str() {
        "read_file" | "list_dir" => {
            if content.is_some() {
                bail!(
                    "line {line}: '{kind}' tool call carried a '{CONTENT_MARKER}' block, which \
                     only '{TOOL_MARKER} write_file' accepts — refusing rather than guessing \
                     which tool was meant."
                );
            }
            if kind == "read_file" {
                Ok(MasonTool::ReadFile { path })
            } else {
                Ok(MasonTool::ListDir { path })
            }
        }
        "write_file" => {
            let Some(body) = content else {
                bail!(
                    "line {line}: 'write_file' tool call for {path:?} has no '{CONTENT_MARKER}' \
                     block, so there is nothing to write. Emit the complete file contents \
                     between '{CONTENT_MARKER}' and '{END_CONTENT_MARKER}'."
                );
            };
            Ok(MasonTool::WriteFile {
                path,
                content: body.join("\n"),
            })
        }
        other => bail!(
            "line {line}: unknown tool {other:?} — the only tools are read_file, list_dir and \
             write_file. Refusing rather than ignoring the call, since ignoring it would look \
             exactly like the model finishing."
        ),
    }
}

/// Render one tool's result in the shape the model was told to expect.
pub fn render_tool_result(tool: &MasonTool, result: &str) -> String {
    format!(
        "### TOOL RESULT: {}\n{result}\n### END TOOL RESULT",
        tool.name()
    )
}

/// Recorded side effect of a proposed write.
///
/// The loop does not own the invocation gateway — that lives on `AppContext`,
/// which this module deliberately does not depend on so the parser stays
/// testable without bootstrapping a whole factory. The orchestrator implements
/// this trait over the same gateway host commands already go through, so tool
/// writes land in `tool_invocations.json` beside them.
#[async_trait::async_trait]
pub trait MasonToolRecorder: Send + Sync {
    /// Record a proposed write. `Ok(false)` means the gateway refused it, which
    /// ends the loop rather than dropping the write.
    async fn record_write(&self, path: &str, byte_len: usize) -> Result<bool>;
}

/// Run the loop with no invocation recording. Used by tests and by any caller
/// with no run context; production always goes through
/// [`run_mason_tool_loop_with_recorder`].
pub async fn run_mason_tool_loop(
    provider: &dyn LlmProvider,
    req: LlmRequest,
    staged: &Path,
    max_turns: u32,
) -> Result<Vec<(String, String)>> {
    run_mason_tool_loop_with_recorder(provider, req, staged, max_turns, None).await
}

/// Drive the model until it stops asking for tools, then hand back what it
/// wants written.
///
/// The returned pairs are `(path, content)` in the order the model settled on
/// them, one entry per distinct file: a later write to a path already written
/// *supersedes* the earlier one rather than colliding with it, because in a
/// sequential loop the second write is the model revising its own work, not two
/// competing edits in one batch. The supersession is reported back to the model
/// in the tool result and each write is recorded separately in the invocation
/// log, so nothing about it is silent — and the batch handed to the caller
/// still has one entry per path, which keeps `validate_mason_edits`'s
/// duplicate-path rejection meaningful for every other transport.
pub async fn run_mason_tool_loop_with_recorder(
    provider: &dyn LlmProvider,
    req: LlmRequest,
    staged: &Path,
    max_turns: u32,
    recorder: Option<&dyn MasonToolRecorder>,
) -> Result<Vec<(String, String)>> {
    let turns = max_turns.max(1);
    // Canonical form of the workspace root, for the "this write targets the
    // root itself" check below. `join_workspace_relative_path` canonicalizes
    // the base it returns, so the two are directly comparable.
    let staged_root = std::fs::canonicalize(staged).unwrap_or_else(|_| staged.to_path_buf());
    let mut messages = req.messages.clone();
    // Keyed by the *resolved* path so two spellings of one file
    // (`js/a.js`, `js/./a.js`) supersede each other here exactly as they would
    // collide on disk, rather than surviving as two entries.
    let mut writes: Vec<(PathBuf, String, String)> = Vec::new();
    let mut last_problem: Option<String> = None;

    for _turn in 0..turns {
        let response = provider
            .complete(LlmRequest {
                messages: messages.clone(),
                max_tokens: req.max_tokens,
                temperature: req.temperature,
            })
            .await?;

        let calls = match parse_tool_calls(&response.content) {
            Ok(calls) => calls,
            Err(error) => {
                // Same contract as the single-shot retry lane: show the model
                // exactly how it failed and ask again. Nothing from this turn
                // is kept — `parse_tool_calls` is all-or-nothing — so no
                // half-read call can leak through.
                let problem = format!("{error:#}");
                messages.push(Message::assistant(response.content.clone()));
                messages.push(Message::user(format!(
                    "Your previous message could not be used: {problem}\n\n{TOOL_LOOP_INSTRUCTION}"
                )));
                last_problem = Some(problem);
                continue;
            }
        };

        // Checked on every turn, not only the terminal one, and before a single
        // call is executed. Two failures live here:
        //
        // - a mixed message, carrying one `write_file` call *and* a `### FILE:`
        //   block for a second file. Executing the call and ignoring the block
        //   applies one file, drops the other, and reports success.
        // - a terminal message that is not terminal at all — a JSON proposal, or
        //   `#### TOOL:` one hash off. Zero calls parse out of it, and without
        //   this check the loop reads it as the model signing off.
        //
        // Either way the whole message is rejected and re-asked.
        if let Some(problem) = unreadable_write_marker(&response.content) {
            messages.push(Message::assistant(response.content.clone()));
            messages.push(Message::user(format!(
                "Your previous message could not be used: {problem}\n\n{TOOL_LOOP_INSTRUCTION}"
            )));
            last_problem = Some(problem);
            continue;
        }

        if calls.is_empty() {
            // No tool calls, and nothing in the message that this lane failed to
            // read. Only now is the model finished.
            return Ok(writes
                .into_iter()
                .map(|(_resolved, path, content)| (path, content))
                .collect());
        }

        let mut results = Vec::new();
        for tool in &calls {
            // Confinement first, for every tool, before anything is read,
            // listed or recorded. `join_workspace_relative_path` is the same
            // function the apply path uses: it rejects absolute paths and any
            // `..` component. A path that escapes ends the run rather than
            // returning an error the model could iterate against — a model
            // reaching outside its workspace has misunderstood its boundary,
            // and the operator should see that immediately.
            //
            // Fed the *normalized* path, because that is what the apply path
            // feeds it. Handing one implementation two different inputs defeats
            // the entire reason for sharing it: `js\..\..\x` has no `..`
            // component on Linux (backslashes are ordinary characters) and
            // sailed through here, only to be rejected at apply time — after
            // the model had been told "recorded write of N bytes" and a
            // `mason_tool_write` record claiming success had been written for a
            // write that could never land.
            //
            // Absolute paths are refused *before* normalizing, because
            // normalizing strips the leading `/` and would quietly reinterpret
            // `/etc/cron.d/evil` as a workspace-relative write. Confined, but
            // not what the model asked for, and not what it was told would
            // happen. Refusing is louder and keeps the instruction honest.
            if Path::new(tool.path()).is_absolute() {
                bail!(
                    "Mason's '{}' tool call uses the absolute path {:?}. Tool paths must be \
                     relative to the staged workspace root.",
                    tool.name(),
                    tool.path()
                );
            }
            let normalized = crate::orchestrator::normalize_project_path(tool.path());
            let resolved = crate::orchestrator::join_workspace_relative_path(staged, &normalized)
                .map_err(|error| {
                anyhow::anyhow!(
                    "Mason's '{}' tool call for {:?} does not resolve inside the staged \
                         workspace: {error:#}",
                    tool.name(),
                    tool.path()
                )
            })?;

            let result = match tool {
                MasonTool::ReadFile { path } => match read_refusal(&normalized) {
                    // Not an escape — the path is inside the workspace — so
                    // this is a soft refusal the model can work around, not a
                    // boundary violation that ends the run.
                    Some(reason) => format!("ERROR: reading {path}: {reason}"),
                    None => match std::fs::read_to_string(&resolved) {
                        Ok(text) => text,
                        // A missing file is information the model asked for, not
                        // a failure of the loop: it is allowed to probe for a
                        // file and learn it is not there.
                        Err(error) => format!("ERROR: reading {path}: {error}"),
                    },
                },
                MasonTool::ListDir { path } => match std::fs::read_dir(&resolved) {
                    Ok(entries) => {
                        let mut names = Vec::new();
                        for entry in entries {
                            // Soft, like the failure to open the directory at
                            // all. A single unreadable entry — a race with
                            // another process, a permission quirk — is not a
                            // reason to end a run that has real writes in it.
                            let entry = match entry {
                                Ok(entry) => entry,
                                Err(error) => {
                                    names.push(format!("(unreadable entry: {error})"));
                                    continue;
                                }
                            };
                            let suffix = if entry.file_type().map(|k| k.is_dir()).unwrap_or(false) {
                                "/"
                            } else {
                                ""
                            };
                            names.push(format!("{}{suffix}", entry.file_name().to_string_lossy()));
                        }
                        names.sort();
                        if names.is_empty() {
                            format!("(empty directory: {path})")
                        } else {
                            names.join("\n")
                        }
                    }
                    Err(error) => format!("ERROR: listing {path}: {error}"),
                },
                MasonTool::WriteFile { path, content } => {
                    // A write that resolves to the workspace root is not a
                    // write. `validate_mason_edits` rejects it at apply time
                    // (its normalized path is empty), so letting it through
                    // here would again promise the model a write that can never
                    // land, and log a successful invocation for it.
                    if resolved == staged_root {
                        bail!(
                            "Mason's write_file call for {path:?} resolves to the workspace root \
                             itself, which is a directory, not a file."
                        );
                    }
                    if let Some(recorder) = recorder {
                        if !recorder.record_write(path, content.len()).await? {
                            bail!(
                                "the invocation gateway refused Mason's tool write to {path:?}. \
                                 No edits were returned."
                            );
                        }
                    }
                    let superseded = writes.iter().position(|(key, _, _)| key == &resolved);
                    match superseded {
                        Some(index) => {
                            let previous = writes[index].2.len();
                            writes[index] = (resolved.clone(), path.clone(), content.clone());
                            format!(
                                "recorded write of {} bytes to {path} (replaces the earlier {} \
                                 byte write to the same file in this loop)",
                                content.len(),
                                previous
                            )
                        }
                        None => {
                            writes.push((resolved.clone(), path.clone(), content.clone()));
                            format!("recorded write of {} bytes to {path}", content.len())
                        }
                    }
                }
            };
            results.push(render_tool_result(tool, &result));
        }

        messages.push(Message::assistant(response.content.clone()));
        messages.push(Message::user(results.join("\n\n")));
    }

    // Partial results are not success. The model asked for a tool on the last
    // turn it had, which means it did not consider itself finished, which means
    // whatever it had written so far is not the set of writes it intended.
    match last_problem {
        Some(problem) => bail!(
            "Mason used all {turns} tool turns without settling on a final set of writes. The \
             last turn was rejected: {problem}"
        ),
        None => bail!(
            "Mason used all {turns} tool turns without settling on a final set of writes — it \
             was still calling tools when the budget ran out. Raise the turn budget or narrow \
             the spec."
        ),
    }
}

/// Why a read is refused, or `None` if it is allowed.
///
/// Before this lane existed, Mason saw a harness-chosen, filtered set of at most
/// eight files (`build_mason_context_files` / `is_mason_context_candidate`).
/// The loop makes the selection *model-chosen*, which is the point — but it also
/// means a `read_file` on `.env` would ship `API_KEY=sk-live-…` verbatim into
/// the provider conversation, from a file no operator ever chose to share.
///
/// Two rules, both narrow:
///
/// - the same blocked prefixes the single-shot context builder already applies,
///   shared as a constant so the two lanes cannot drift apart;
/// - credential-bearing filenames, which the prefix list does not cover because
///   they sit at the workspace root.
///
/// This is a filter, not a boundary: refusing a read is reported to the model as
/// a normal tool error and it can carry on. Nothing the model produced is
/// discarded, so the loud-failure invariant does not apply here.
fn read_refusal(normalized: &str) -> Option<String> {
    for prefix in crate::orchestrator::MASON_BLOCKED_PATH_PREFIXES {
        if normalized.starts_with(prefix) {
            return Some(format!(
                "{prefix} is excluded from Mason's reads — it holds build output or factory \
                 state, not product source"
            ));
        }
    }

    let file_name = Path::new(normalized)
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let secret = file_name == ".env"
        || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name == "credentials.json"
        || file_name == "credentials"
        || file_name == ".netrc"
        || file_name == ".npmrc"
        || file_name == ".pypirc"
        || file_name == "id_rsa"
        || file_name == "id_ed25519";
    if secret {
        return Some(format!(
            "{file_name} holds credentials and is never shared with a model. Ask the operator \
             for any value you need from it."
        ));
    }

    None
}

/// Detects a message that is trying to write files in a way this lane cannot
/// read — which is the difference between a model that has finished and a model
/// whose output was thrown away.
///
/// This is the guard that makes termination *positive*. `parse_tool_calls`
/// returning zero calls only means no exact `### TOOL:` header was found, and
/// there are far more ways to miss that header than to hit it:
///
/// - another live Mason transport (`### FILE:`, `### PATCH:`, a JSON edit
///   proposal) — JSON is the likeliest, since it is still a supported transport
///   and several providers revert to it under pressure;
/// - the right vocabulary one character off — `#### TOOL:` (four hashes),
///   `### Tool:` (case), `## CONTENT`. This is the sharpest case: the model used
///   exactly the vocabulary it was taught, and without this check its write is
///   read as it signing off.
///
/// Every one of those parses to zero calls. Treating that as "finished" drops
/// the write and reports the run applied. So a near-miss marker rejects the
/// message and re-asks instead.
///
/// Block-aware on purpose, and exact markers are explicitly allowed through. A
/// `### FILE:` line inside a `write_file` content block is not a foreign
/// transport — it is the file's own text, and this repo contains several files
/// that legitimately document those markers. A naive substring scan would
/// reject such a write forever, on every retry, which is the mistake
/// `has_top_level_patch_header` was fixed for in the patch lane.
///
/// Only called after `parse_tool_calls` has already succeeded, so the block
/// structure is known to be well-formed and this toggle cannot desynchronize
/// from the parser's.
fn unreadable_write_marker(raw: &str) -> Option<String> {
    let mut in_content = false;
    for (index, line) in raw.split('\n').enumerate() {
        let trimmed = line.trim_end_matches('\r').trim();
        if in_content {
            if trimmed == END_CONTENT_MARKER {
                in_content = false;
            }
            continue;
        }
        if trimmed == CONTENT_MARKER {
            in_content = true;
            continue;
        }
        // The exact markers this lane *does* read are fine — they are why the
        // message parsed. Only near-misses and foreign transports get here.
        if trimmed.starts_with(TOOL_MARKER) || trimmed == END_CONTENT_MARKER {
            continue;
        }
        if let Some(word) = near_miss_marker_word(trimmed) {
            return Some(format!(
                "line {}: {trimmed:?} looks like a '{word}' marker but is not one — this lane \
                 reads only '{TOOL_MARKER}', '{CONTENT_MARKER}' and '{END_CONTENT_MARKER}', \
                 spelled exactly, with three hashes and in upper case",
                index + 1
            ));
        }
        if looks_like_json_edit_proposal(trimmed) {
            return Some(format!(
                "line {}: {trimmed:?} looks like a JSON edit proposal. That transport is not \
                 read in the tool loop — every write must be a '{TOOL_MARKER} write_file' call",
                index + 1
            ));
        }
    }
    None
}

/// Matches `^#{2,6}\s*(TOOL|CONTENT|END CONTENT|FILE|PATCH|END FILE)`,
/// case-insensitively, without pulling in a regex dependency for six literals.
fn near_miss_marker_word(trimmed: &str) -> Option<&'static str> {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(2..=6).contains(&hashes) {
        return None;
    }
    let rest = trimmed[hashes..].trim_start().to_ascii_uppercase();
    // Longest first: "END CONTENT" must not be shadowed by a prefix match, and
    // "END FILE" must not be read as "FILE".
    for word in [
        "END CONTENT",
        "END FILE",
        "CONTENT",
        "TOOL",
        "FILE",
        "PATCH",
    ] {
        let Some(after) = rest.strip_prefix(word) else {
            continue;
        };
        // Word boundary, or `## Files changed` in an ordinary summary heading
        // reads as a mangled `### FILE:` and gets the message rejected — every
        // turn, until the budget is gone. A marker is followed by a colon, a
        // space or nothing; never by more letters.
        if after
            .chars()
            .next()
            .is_none_or(|next| !next.is_alphanumeric() && next != '_')
        {
            return Some(word);
        }
    }
    None
}

/// The shape of the JSON edit lane, which is still a live Mason transport: an
/// `"edits"` array of `{"path": ..., "content": ...}` objects. Matching the two
/// keys is enough — a false positive costs one re-asked turn, and a false
/// negative costs the operator a file.
fn looks_like_json_edit_proposal(trimmed: &str) -> bool {
    trimmed.contains("\"edits\"") || trimmed.contains("\"path\":")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct ScriptedProvider {
        responses: Mutex<Vec<String>>,
        seen: Mutex<Vec<Vec<Message>>>,
    }

    impl ScriptedProvider {
        fn new(responses: &[&str]) -> Self {
            Self {
                responses: Mutex::new(responses.iter().map(|r| r.to_string()).collect()),
                seen: Mutex::new(Vec::new()),
            }
        }
        fn calls(&self) -> usize {
            self.seen.lock().expect("lock").len()
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for ScriptedProvider {
        async fn complete(&self, req: LlmRequest) -> Result<crate::llm::LlmResponse> {
            self.seen.lock().expect("lock").push(req.messages.clone());
            let mut responses = self.responses.lock().expect("lock");
            let content = if responses.is_empty() {
                // A model that never stops asking for tools.
                "### TOOL: list_dir\npath: .\n".to_string()
            } else {
                responses.remove(0)
            };
            Ok(crate::llm::LlmResponse {
                content,
                usage: None,
            })
        }
    }

    fn request() -> LlmRequest {
        LlmRequest {
            messages: vec![Message::user("do the work")],
            max_tokens: 1000,
            temperature: 0.1,
        }
    }

    #[test]
    fn tool_calls_parse_from_the_wire_format() {
        assert_eq!(
            parse_tool_call("### TOOL: read_file\npath: js/data.js\n"),
            Some(MasonTool::ReadFile {
                path: "js/data.js".to_string()
            })
        );
        assert_eq!(
            parse_tool_call("### TOOL: list_dir\npath: js\n"),
            Some(MasonTool::ListDir {
                path: "js".to_string()
            })
        );
        assert_eq!(parse_tool_call("SUMMARY: done\n"), None);
    }

    #[test]
    fn write_file_tool_carries_its_content_block() {
        let raw = "### TOOL: write_file\npath: js/bonus.js\n### CONTENT\nG.rooms.bonus = {};\n### END CONTENT\n";
        let Some(MasonTool::WriteFile { path, content }) = parse_tool_call(raw) else {
            panic!("expected a write_file call");
        };
        assert_eq!(path, "js/bonus.js");
        assert_eq!(content.trim(), "G.rooms.bonus = {};");
    }

    #[test]
    fn a_message_with_no_tool_header_is_terminal_not_an_error() {
        let calls = parse_tool_calls("SUMMARY: done\nI wrote everything already.\n")
            .expect("prose is not an error");
        assert!(calls.is_empty(), "no header means the model is finished");
    }

    #[test]
    fn several_tool_calls_in_one_message_all_survive() {
        let raw = "\
### TOOL: read_file
path: js/data.js
### TOOL: write_file
path: js/a.js
### CONTENT
a
### END CONTENT
### TOOL: list_dir
path: js
";
        let calls = parse_tool_calls(raw).expect("three calls must parse");
        assert_eq!(
            calls,
            vec![
                MasonTool::ReadFile {
                    path: "js/data.js".to_string()
                },
                MasonTool::WriteFile {
                    path: "js/a.js".to_string(),
                    content: "a".to_string()
                },
                MasonTool::ListDir {
                    path: "js".to_string()
                },
            ],
            "a second call must not overwrite the first"
        );
    }

    #[test]
    fn an_unknown_tool_name_is_refused_rather_than_read_as_finishing() {
        let error = parse_tool_calls("### TOOL: delete_file\npath: js/a.js\n")
            .expect_err("an unknown tool must not parse");
        let message = format!("{error:#}");
        assert!(
            message.contains("delete_file"),
            "the error must name the tool: {message}"
        );
    }

    #[test]
    fn a_tool_call_without_a_path_is_refused() {
        let error =
            parse_tool_calls("### TOOL: read_file\nI forgot the path\n").expect_err("no path");
        assert!(format!("{error:#}").contains("missing its 'path:' line"));
    }

    #[test]
    fn a_write_without_a_content_block_is_refused() {
        let error =
            parse_tool_calls("### TOOL: write_file\npath: js/a.js\n").expect_err("no content");
        assert!(format!("{error:#}").contains("nothing to write"));
    }

    #[test]
    fn an_empty_content_block_writes_an_empty_file() {
        let call =
            parse_tool_call("### TOOL: write_file\npath: js/a.js\n### CONTENT\n### END CONTENT\n")
                .expect("an empty file is a legal write");
        assert_eq!(
            call,
            MasonTool::WriteFile {
                path: "js/a.js".to_string(),
                content: String::new()
            }
        );
    }

    #[test]
    fn an_unclosed_content_block_is_refused_rather_than_truncated() {
        let raw = "### TOOL: write_file\npath: js/a.js\n### CONTENT\nline one\nline two\n";
        let error = parse_tool_calls(raw).expect_err("a cut-off write must not parse");
        let message = format!("{error:#}");
        assert!(
            message.contains("never closed"),
            "the error must say the block never closed: {message}"
        );
    }

    #[test]
    fn a_tool_header_inside_an_open_content_block_is_refused() {
        // The failure this guard exists for: without it, the second call and
        // every line after it is swallowed into the first file's content, and
        // the loop reports success having written a corrupted file and dropped
        // a write entirely.
        let raw = "\
### TOOL: write_file
path: js/a.js
### CONTENT
a
### TOOL: write_file
path: js/b.js
### CONTENT
b
### END CONTENT
";
        let error = parse_tool_calls(raw).expect_err("a missing closer must not parse");
        assert!(format!("{error:#}").contains("still open"));
    }

    #[test]
    fn a_stray_end_content_marker_is_refused() {
        let error = parse_tool_calls("### TOOL: read_file\npath: js/a.js\n### END CONTENT\n")
            .expect_err("a stray closer must not parse");
        assert!(format!("{error:#}").contains("stray"));
    }

    #[test]
    fn a_content_block_on_a_read_call_is_refused() {
        let raw = "### TOOL: read_file\npath: js/a.js\n### CONTENT\nx\n### END CONTENT\n";
        let error = parse_tool_calls(raw).expect_err("read_file takes no content");
        assert!(format!("{error:#}").contains("only '### TOOL: write_file' accepts"));
    }

    #[test]
    fn tool_calls_and_prose_coexist() {
        let raw = "\
Let me look at the data file first.

### TOOL: read_file
path: js/data.js

I will decide what to write once I see it.
";
        assert_eq!(
            parse_tool_calls(raw).expect("prose around a call is fine"),
            vec![MasonTool::ReadFile {
                path: "js/data.js".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn the_loop_reads_then_writes_and_returns_the_pairs() {
        let staged = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(staged.path().join("js")).expect("mkdir");
        std::fs::write(staged.path().join("js/data.js"), "G.rooms = {};").expect("write");

        let provider = ScriptedProvider::new(&[
            "### TOOL: read_file\npath: js/data.js\n",
            "### TOOL: write_file\npath: js/bonus.js\n### CONTENT\nG.rooms.bonus = {};\n### END CONTENT\n",
            "SUMMARY: added the bonus room\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 6)
            .await
            .expect("the loop must finish");

        assert_eq!(
            writes,
            vec![("js/bonus.js".to_string(), "G.rooms.bonus = {};".to_string())]
        );
        assert_eq!(provider.calls(), 3);

        // The write is returned, never performed: the caller owns the one apply
        // path, and a second one is how a safety check gets forgotten.
        assert!(
            !staged.path().join("js/bonus.js").exists(),
            "the loop must not touch the workspace"
        );

        // The file it read must have reached it.
        let seen = provider.seen.lock().expect("lock");
        let second_turn = &seen[1];
        assert!(
            second_turn
                .iter()
                .any(|m| m.content.contains("G.rooms = {};")),
            "the read result must be fed back to the model"
        );
    }

    #[tokio::test]
    async fn a_path_escaping_the_workspace_ends_the_loop() {
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&["### TOOL: read_file\npath: ../../etc/passwd\n"]);

        let error = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect_err("an escaping path must end the run");
        let message = format!("{error:#}");
        assert!(
            message.contains("staged") && message.contains("passwd"),
            "the error must name the offending path: {message}"
        );
    }

    #[tokio::test]
    async fn an_absolute_path_is_refused_too() {
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: /etc/cron.d/evil\n### CONTENT\nboom\n### END CONTENT\n",
        ]);

        let error = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect_err("an absolute path must end the run");
        assert!(format!("{error:#}").contains("absolute"));
    }

    #[tokio::test]
    async fn a_write_that_escapes_is_refused_before_it_is_recorded() {
        struct CountingRecorder {
            writes: Mutex<Vec<String>>,
        }
        #[async_trait::async_trait]
        impl MasonToolRecorder for CountingRecorder {
            async fn record_write(&self, path: &str, _byte_len: usize) -> Result<bool> {
                self.writes.lock().expect("lock").push(path.to_string());
                Ok(true)
            }
        }

        let staged = tempfile::tempdir().expect("tempdir");
        let recorder = CountingRecorder {
            writes: Mutex::new(Vec::new()),
        };
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: ../escape.js\n### CONTENT\nx\n### END CONTENT\n",
        ]);

        run_mason_tool_loop_with_recorder(&provider, request(), staged.path(), 4, Some(&recorder))
            .await
            .expect_err("an escaping write must end the run");
        assert!(
            recorder.writes.lock().expect("lock").is_empty(),
            "confinement must run before the gateway, not after"
        );
    }

    #[tokio::test]
    async fn a_model_that_never_finishes_fails_rather_than_returning_partial_writes() {
        let staged = tempfile::tempdir().expect("tempdir");
        // One real write, then an endless list_dir (the scripted provider's
        // fallback), so there *is* a partial result available to return.
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n",
        ]);

        let error = run_mason_tool_loop(&provider, request(), staged.path(), 3)
            .await
            .expect_err("an unfinished loop must not report success");
        let message = format!("{error:#}");
        assert!(
            message.contains("without settling on a final set of writes"),
            "the exhaustion message must say what went wrong: {message}"
        );
        assert_eq!(provider.calls(), 3, "the turn budget must be honoured");
    }

    #[tokio::test]
    async fn a_malformed_tool_call_is_fed_back_and_the_loop_recovers() {
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            // Unknown tool: the failure that, if parsed as `None`, would look
            // exactly like the model finishing with no writes at all.
            "### TOOL: delete_file\npath: js/a.js\n",
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 5)
            .await
            .expect("the loop must recover after the correction");
        assert_eq!(writes, vec![("js/a.js".to_string(), "a".to_string())]);

        let seen = provider.seen.lock().expect("lock");
        assert!(
            seen[1]
                .iter()
                .any(|m| m.content.contains("could not be used")
                    && m.content.contains("delete_file")),
            "the model must be told exactly what it got wrong"
        );
    }

    #[tokio::test]
    async fn a_file_block_in_the_final_message_is_not_read_as_finishing() {
        // The silent-skip this guard exists for: the model reverts to the
        // single-shot transport, the loop sees no tool call, and every file in
        // those blocks is dropped while the run reports success.
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "SUMMARY: done\n### FILE: js/a.js\nvar a = 1;\n### END FILE\n",
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\nvar a = 1;\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 5)
            .await
            .expect("the loop must recover");
        assert_eq!(
            writes,
            vec![("js/a.js".to_string(), "var a = 1;".to_string())]
        );
    }

    #[tokio::test]
    async fn a_file_block_beside_a_tool_call_rejects_the_whole_message() {
        // The nastier half of the same failure: the model emits one write_file
        // call *and* a `### FILE:` block for a second file. Executing the call
        // and ignoring the block would apply one file, drop the other, and
        // report success — the operator would never learn the second file was
        // never written.
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n\
             ### FILE: js/b.js\nb\n### END FILE\n",
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n\
             ### TOOL: write_file\npath: js/b.js\n### CONTENT\nb\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 5)
            .await
            .expect("the loop must recover");
        assert_eq!(
            writes,
            vec![
                ("js/a.js".to_string(), "a".to_string()),
                ("js/b.js".to_string(), "b".to_string()),
            ],
            "the mixed message must be re-asked, not half-applied"
        );
    }

    /// Every one of these returned `Ok([("js/a.js", "a")])` with the second
    /// file silently gone, and the run reporting `applied`, when termination was
    /// merely "no `### TOOL:` header, and no exact `### FILE:`/`### PATCH:`".
    /// The `#### TOOL:` case is the sharpest: the model uses the exact
    /// vocabulary it was taught, one character off, and its write is read as it
    /// signing off. The JSON case is the likeliest, since JSON is still a live
    /// Mason transport several providers revert to.
    #[tokio::test]
    async fn a_terminal_message_carrying_an_unreadable_write_is_never_read_as_finishing() {
        let cases: &[(&str, &str)] = &[
            (
                "json edit proposal",
                "{\"summary\":\"s\",\"edits\":[{\"path\":\"js/b.js\",\"content\":\"b\"}]}",
            ),
            ("four-hash FILE", "#### FILE: js/b.js\nb\n#### END FILE"),
            ("wrong-case FILE", "### File: js/b.js\nb\n### End File"),
            (
                "four-hash TOOL",
                "#### TOOL: write_file\npath: js/b.js\n#### CONTENT\nb\n#### END CONTENT",
            ),
            (
                "wrong-case TOOL",
                "### Tool: write_file\npath: js/b.js\n### Content\nb\n### End Content",
            ),
            ("two-hash PATCH", "## PATCH: js/b.js\n<<<<<<< SEARCH"),
        ];

        for (label, terminal) in cases {
            let staged = tempfile::tempdir().expect("tempdir");
            let provider = ScriptedProvider::new(&[
                "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n",
                terminal,
                "### TOOL: write_file\npath: js/b.js\n### CONTENT\nb\n### END CONTENT\n",
                "SUMMARY: done\n",
            ]);

            let writes = run_mason_tool_loop(&provider, request(), staged.path(), 6)
                .await
                .unwrap_or_else(|error| panic!("{label}: loop must recover: {error:#}"));
            assert_eq!(
                writes,
                vec![
                    ("js/a.js".to_string(), "a".to_string()),
                    ("js/b.js".to_string(), "b".to_string()),
                ],
                "{label}: the unreadable write must be re-asked, not read as finishing"
            );
        }
    }

    #[test]
    fn plain_prose_and_markdown_headings_still_terminate() {
        // The other side of the guard: rejecting too much would strand a model
        // that really is finished, burning its whole turn budget on a message
        // that was correct.
        for terminal in [
            "SUMMARY: done\n",
            "I have written both files. Nothing else is needed.\n",
            "## Summary\n\nAdded the bonus room and wired it up.\n",
            "### Notes\n\n- the room id is `bonus`\n",
            // The near-miss matcher must not read ordinary headings as mangled
            // markers, or a model that writes a normal summary is rejected
            // every turn until its budget is gone.
            "## Files changed\n\n- js/bonus.js\n",
            "### Patching notes\n\nNone needed.\n",
            "#### Toolchain\n\nNo change.\n",
            "### Contents of the room\n\nA lamp.\n",
        ] {
            assert_eq!(
                unreadable_write_marker(terminal),
                None,
                "must terminate cleanly: {terminal:?}"
            );
        }
    }

    #[test]
    fn a_near_miss_marker_inside_written_content_is_not_a_near_miss() {
        // Block-awareness has to survive the looser matcher too, or Mason can
        // never write a file that documents these formats — this repo's own
        // `mason_transport.rs` being the obvious example.
        let raw = "### TOOL: write_file\npath: docs/f.md\n### CONTENT\n\
                   #### TOOL: write_file\n#### FILE: x\n{\"edits\":[]}\n### END CONTENT\n";
        assert_eq!(unreadable_write_marker(raw), None);
    }

    #[test]
    fn an_indented_end_content_marker_closes_the_block_and_the_instruction_says_so() {
        // The parser trims before comparing, symmetrically with
        // `collect_fenced_edits`, so a markdown-indented marker DOES end the
        // write and everything after it is lost. That behaviour is deliberate
        // and shared; the fix is that the model is now warned about it rather
        // than left to discover it by losing a file.
        let raw =
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\nkept\n    ### END CONTENT\nlost\n";
        let call = parse_tool_call(raw).expect("the indented marker closes the block");
        assert_eq!(
            call,
            MasonTool::WriteFile {
                path: "js/a.js".to_string(),
                content: "kept".to_string()
            },
            "indentation does not exempt a marker — pinning the parser's actual behaviour"
        );

        assert!(
            TOOL_LOOP_INSTRUCTION.contains("Indenting such a line does NOT make it safe"),
            "the instruction must warn about the indented case, since the parser cannot"
        );
        assert!(
            TOOL_LOOP_INSTRUCTION.contains("only non-whitespace text"),
            "the instruction must describe the trim-then-compare rule, not 'consists solely of'"
        );
    }

    #[tokio::test]
    async fn credential_files_are_refused_and_the_refusal_is_told_to_the_model() {
        let staged = tempfile::tempdir().expect("tempdir");
        std::fs::write(staged.path().join(".env"), "API_KEY=sk-live-xyz").expect("write");
        std::fs::create_dir_all(staged.path().join("target")).expect("mkdir");
        std::fs::write(staged.path().join("target/build.log"), "noise").expect("write");

        let provider = ScriptedProvider::new(&[
            "### TOOL: read_file\npath: .env\n",
            "### TOOL: read_file\npath: target/build.log\n",
            "SUMMARY: nothing to do\n",
        ]);

        run_mason_tool_loop(&provider, request(), staged.path(), 5)
            .await
            .expect("a refused read is not a failure");

        let seen = provider.seen.lock().expect("lock");
        let after_env = seen[1]
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        assert!(
            after_env.contains("credentials") && !after_env.contains("sk-live-xyz"),
            "the secret must never reach the conversation: {after_env}"
        );
        let after_target = seen[2]
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        assert!(
            after_target.contains("excluded from Mason's reads"),
            "the shared blocked-prefix list must apply to reads too: {after_target}"
        );
    }

    #[test]
    fn ordinary_source_reads_are_not_refused() {
        assert_eq!(read_refusal("js/data.js"), None);
        assert_eq!(read_refusal("src/lib.rs"), None);
        assert_eq!(read_refusal("environment/setup.md"), None);
        assert!(read_refusal(".env.local").is_some());
        assert!(read_refusal("certs/server.pem").is_some());
        assert!(read_refusal("node_modules/left-pad/index.js").is_some());
    }

    #[tokio::test]
    async fn a_backslash_escape_is_refused_by_the_loop_not_by_the_apply_path() {
        // Backslashes are ordinary characters on Linux, so `js\..\..\x` has no
        // `Component::ParentDir` and sailed through confinement — until the
        // apply path normalized it to `js/../../x` and rejected it, long after
        // the model had been told the write was recorded and a successful
        // invocation had been logged for it.
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js\\..\\..\\x\n### CONTENT\nx\n### END CONTENT\n",
        ]);

        let error = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect_err("the loop must refuse what the apply path would refuse");
        assert!(format!("{error:#}").contains("does not resolve inside the staged workspace"));
    }

    #[tokio::test]
    async fn a_write_to_the_workspace_root_is_refused() {
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: .\n### CONTENT\nx\n### END CONTENT\n",
        ]);

        let error = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect_err("a write with no filename can never land");
        assert!(format!("{error:#}").contains("workspace root"));
    }

    #[tokio::test]
    async fn listing_the_workspace_root_is_still_allowed() {
        // The root check is scoped to writes on purpose: `list_dir .` is the
        // loop's most useful first move.
        let staged = tempfile::tempdir().expect("tempdir");
        std::fs::write(staged.path().join("index.html"), "x").expect("write");
        let provider = ScriptedProvider::new(&["### TOOL: list_dir\npath: .\n", "SUMMARY: seen\n"]);

        run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect("listing the root must work");
        let seen = provider.seen.lock().expect("lock");
        assert!(seen[1]
            .last()
            .map(|m| m.content.contains("index.html"))
            .unwrap_or(false));
    }

    #[tokio::test]
    async fn a_file_marker_inside_written_content_is_not_a_foreign_transport() {
        // Mason writing documentation about the fenced transport — which the
        // files in this very repo do. A substring scan would reject this write
        // forever, on every retry, for containing text it was asked to write.
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: docs/format.md\n### CONTENT\n\
             Emit blocks like:\n### FILE: path\ncontent\n### END FILE\n### END CONTENT\n",
            "SUMMARY: documented the format\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect("documenting a marker is a legal write");
        assert_eq!(writes.len(), 1);
        assert!(
            writes[0].1.contains("### FILE: path"),
            "the marker must survive into the written content: {:?}",
            writes[0].1
        );
        assert_eq!(
            provider.calls(),
            2,
            "the write must be accepted on the first try, not re-asked"
        );
    }

    #[tokio::test]
    async fn a_second_write_to_one_path_supersedes_rather_than_duplicating() {
        // Sent to the caller as one entry per file, because two entries for one
        // path is exactly what `validate_mason_edits` refuses — and here the
        // model's intent is unambiguous: the later write is the revision.
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\nfirst\n### END CONTENT\n",
            "### TOOL: write_file\npath: js/./a.js\n### CONTENT\nsecond\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 5)
            .await
            .expect("the loop must finish");
        assert_eq!(
            writes,
            vec![("js/./a.js".to_string(), "second".to_string())],
            "one entry per real file, carrying the latest content"
        );

        let seen = provider.seen.lock().expect("lock");
        assert!(
            seen[2]
                .iter()
                .any(|m| m.content.contains("replaces the earlier")),
            "supersession must be reported back, never silent"
        );
    }

    #[tokio::test]
    async fn a_refused_gateway_write_ends_the_loop() {
        struct RefusingRecorder;
        #[async_trait::async_trait]
        impl MasonToolRecorder for RefusingRecorder {
            async fn record_write(&self, _path: &str, _byte_len: usize) -> Result<bool> {
                Ok(false)
            }
        }

        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\na\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        let error = run_mason_tool_loop_with_recorder(
            &provider,
            request(),
            staged.path(),
            5,
            Some(&RefusingRecorder),
        )
        .await
        .expect_err("a refused write must not be returned as an edit");
        assert!(format!("{error:#}").contains("gateway refused"));
    }

    #[tokio::test]
    async fn every_write_reaches_the_invocation_gateway() {
        struct CountingRecorder {
            writes: Mutex<Vec<(String, usize)>>,
        }
        #[async_trait::async_trait]
        impl MasonToolRecorder for CountingRecorder {
            async fn record_write(&self, path: &str, byte_len: usize) -> Result<bool> {
                self.writes
                    .lock()
                    .expect("lock")
                    .push((path.to_string(), byte_len));
                Ok(true)
            }
        }

        let staged = tempfile::tempdir().expect("tempdir");
        let recorder = CountingRecorder {
            writes: Mutex::new(Vec::new()),
        };
        let provider = ScriptedProvider::new(&[
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\naa\n### END CONTENT\n",
            "### TOOL: write_file\npath: js/a.js\n### CONTENT\nbbb\n### END CONTENT\n",
            "SUMMARY: done\n",
        ]);

        run_mason_tool_loop_with_recorder(&provider, request(), staged.path(), 5, Some(&recorder))
            .await
            .expect("the loop must finish");

        assert_eq!(
            *recorder.writes.lock().expect("lock"),
            vec![("js/a.js".to_string(), 2), ("js/a.js".to_string(), 3)],
            "a superseded write is still a write that happened, and must be logged"
        );
    }

    #[tokio::test]
    async fn a_missing_file_is_reported_to_the_model_not_fatal() {
        let staged = tempfile::tempdir().expect("tempdir");
        let provider = ScriptedProvider::new(&[
            "### TOOL: read_file\npath: js/absent.js\n",
            "SUMMARY: nothing to do\n",
        ]);

        let writes = run_mason_tool_loop(&provider, request(), staged.path(), 4)
            .await
            .expect("a missing file is information, not a failure");
        assert!(writes.is_empty());

        let seen = provider.seen.lock().expect("lock");
        assert!(seen[1].iter().any(|m| m.content.contains("ERROR: reading")));
    }

    #[test]
    fn the_tool_loop_is_off_unless_a_spec_asks_for_it() {
        // The whole safety argument for this module rests on this: no existing
        // spec sets `tool_loop`, so no existing run reaches any of the code
        // above. If `#[serde(default)]` ever came off the field, a spec without
        // it would fail to load instead of defaulting to the single-shot lane.
        let existing: crate::models::WorkerHarnessConfig =
            serde_yaml::from_str("adapter: harkonnen\nllm_edits: true\ngit_branch: true\n")
                .expect("a spec written before this feature must still load");
        assert!(
            !existing.tool_loop,
            "a spec that never heard of the tool loop must not run it"
        );
        assert!(!crate::models::WorkerHarnessConfig::default().tool_loop);

        let opted_in: crate::models::WorkerHarnessConfig =
            serde_yaml::from_str("adapter: harkonnen\nllm_edits: true\ntool_loop: true\n")
                .expect("opting in must parse");
        assert!(
            opted_in.tool_loop,
            "and a spec that asks for it must get it"
        );
    }

    #[test]
    fn tool_results_render_in_the_shape_the_model_was_promised() {
        let tool = MasonTool::ListDir {
            path: "js".to_string(),
        };
        assert_eq!(
            render_tool_result(&tool, "a.js\nb.js"),
            "### TOOL RESULT: list_dir\na.js\nb.js\n### END TOOL RESULT"
        );
    }
}
