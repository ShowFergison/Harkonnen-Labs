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
- You may emit more than one tool call in a single message; each is executed in \
order and every result is returned to you.
- write_file replaces the whole file. Include the complete contents, exactly as \
they should appear on disk. Do not escape quotes, backslashes or newlines, and \
do not wrap contents in backticks.
- File content must not contain a line consisting solely of '### END CONTENT', \
and must not contain a line starting with '### TOOL:' — the parser cannot \
distinguish those from real markers.
- Do NOT use '### FILE:' or '### PATCH:' blocks in this mode. They are not read \
here, and a message containing one is rejected.
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
/// `Ok(vec![])` means exactly one thing: the message contains no `### TOOL:`
/// header, so the model is done. Every other reading of the input — an unknown
/// tool name, a missing `path:`, a `write_file` with no content block, a
/// content block that never closes, a second `path:` for one call — is an
/// `Err`. None of them are recoverable by guessing, and every guess would drop
/// or invent a file write.
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

        // Checked on every turn, not only on the terminal one, and before a
        // single call is executed. A model reverting to the single-shot
        // transport does not always do it cleanly: a message can carry one
        // `write_file` call *and* a `### FILE:` block for a second file, and
        // accepting that message would apply the first file while the second
        // vanished without a word — the run reporting success either way. The
        // whole message is rejected and re-asked instead.
        if let Some(marker) = foreign_transport_marker(&response.content) {
            let problem = format!(
                "the message contained a '{marker}' block, which the tool loop does not read. \
                 Every write must be a '{TOOL_MARKER} write_file' call."
            );
            messages.push(Message::assistant(response.content.clone()));
            messages.push(Message::user(format!(
                "Your previous message could not be used: {problem}\n\n{TOOL_LOOP_INSTRUCTION}"
            )));
            last_problem = Some(problem);
            continue;
        }

        if calls.is_empty() {
            // No tool calls and no foreign markers: the model is finished.
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
            let resolved = crate::orchestrator::join_workspace_relative_path(staged, tool.path())
                .map_err(|error| {
                anyhow::anyhow!(
                    "Mason's '{}' tool call for {:?} does not resolve inside the staged \
                         workspace: {error:#}",
                    tool.name(),
                    tool.path()
                )
            })?;

            let result = match tool {
                MasonTool::ReadFile { path } => match std::fs::read_to_string(&resolved) {
                    Ok(text) => text,
                    // A missing file is information the model asked for, not a
                    // failure of the loop: it is allowed to probe for a file
                    // and learn it is not there.
                    Err(error) => format!("ERROR: reading {path}: {error}"),
                },
                MasonTool::ListDir { path } => match std::fs::read_dir(&resolved) {
                    Ok(entries) => {
                        let mut names = Vec::new();
                        for entry in entries {
                            let entry = entry?;
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

/// Detects a write expressed in one of the *other* Mason transports.
///
/// Block-aware on purpose. A `### FILE:` line inside a `write_file` content
/// block is not a foreign transport — it is the file's own text, and this repo
/// contains several files that legitimately document that exact marker. A naive
/// substring scan would reject those writes forever, on every retry, which is
/// the mistake `has_top_level_patch_header` was fixed for in the patch lane.
/// Only a header at top level, outside any content block, counts.
///
/// Only called after `parse_tool_calls` has already succeeded, so the block
/// structure is known to be well-formed and this toggle cannot desynchronize.
fn foreign_transport_marker(raw: &str) -> Option<&'static str> {
    let mut in_content = false;
    for line in raw.split('\n') {
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
        if trimmed.starts_with("### FILE:") {
            return Some("### FILE:");
        }
        if trimmed.starts_with("### PATCH:") {
            return Some("### PATCH:");
        }
    }
    None
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
