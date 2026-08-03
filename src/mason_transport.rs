use anyhow::{bail, Result};

/// Wording handed to the model. Lives next to the parser deliberately: a prompt
/// that asks for one format while the parser expects another is precisely the
/// class of failure this transport exists to eliminate.
pub const FENCED_FORMAT_INSTRUCTION: &str = "\
Respond in this exact plain-text format. Do NOT use JSON. Do NOT use markdown code fences.

SUMMARY: <one line describing the change>
RATIONALE:
- <one reason per line>

Then, for every file you are writing, a block of exactly this shape:

### FILE: <relative/path/from/the/workspace/root>
<the complete contents of the file, written literally>
### END FILE

Write file contents exactly as they should appear on disk. Do not escape \
quotes, backslashes or newlines. Do not wrap contents in backticks. Emit one \
block per file, and nothing after the final ### END FILE. Important: file content \
must not contain any line that starts with '### FILE:' — the parser cannot \
distinguish such a line from a real file header. Also, no line of content may \
consist solely of '### END FILE' — the parser uses that exact phrase to mark block ends.";

const FILE_MARKER: &str = "### FILE:";
const END_MARKER: &str = "### END FILE";
const PATCH_HEADER_MARKER: &str = "### PATCH:";
const REPLACE_END_MARKER: &str = ">>>>>>> REPLACE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FencedFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FencedEnvelope {
    pub summary: String,
    pub rationale: Vec<String>,
    pub files: Vec<FencedFile>,
}

impl FencedEnvelope {
    /// `action` is always `write` — the fenced transport only expresses whole
    /// files, which is exactly what the existing apply path already handles.
    pub fn into_edits(self) -> (String, Vec<String>, Vec<(String, String)>) {
        let files = self
            .files
            .into_iter()
            .map(|file| (file.path, file.content))
            .collect();
        (self.summary, self.rationale, files)
    }
}

/// Tracks whether the line currently being scanned lies inside a `### FILE:`
/// … `### END FILE` or `### PATCH:` … `>>>>>>> REPLACE` block body, as opposed
/// to being a header/closer line or ordinary top-level text. Each block only
/// closes on the marker that actually opened it — a `>>>>>>> REPLACE` line
/// quoted inside a `### FILE:` block's documentation content does not
/// prematurely end that FILE block, and vice versa.
///
/// This is a best-effort scanner, not a structural validator: `parse_fenced_edits`
/// and `parse_patch_blocks` remain the source of truth for whether the blocks
/// themselves are well-formed. Shared by three consumers —
/// `parse_summary_and_rationale`, `has_top_level_patch_header`, and
/// `parse_patch_blocks` — one block-tracking implementation, so none of them
/// can disagree about what counts as "inside a block." `parse_patch_blocks`
/// uses `file_blocks_only()`: it already owns `### PATCH:` block internals
/// via its own SEARCH/REPLACE section state machine, so this tracker must
/// not also treat `### PATCH:` as an opener for it — doing so would cause a
/// *real* top-level patch's own body lines to be skipped here as if they
/// were block content, when they need to reach that state machine instead.
///
/// Coupling note: this tracker and `parse_fenced_edits` agree on the FILE
/// block closing rule only because both hardcode the literal `### END FILE`
/// (via the shared `END_MARKER` constant). Nothing besides that shared
/// constant enforces the agreement — if `parse_fenced_edits`'s own
/// file-block-building loop ever changes what it accepts as a closer, this
/// tracker must change with it.
struct BlockTracker {
    closing_marker: Option<&'static str>,
    recognize_patch_headers: bool,
}

impl BlockTracker {
    /// Tracks both `### FILE:` and `### PATCH:` blocks — for consumers that
    /// need to skip over either kind of block indiscriminately.
    fn new() -> Self {
        Self {
            closing_marker: None,
            recognize_patch_headers: true,
        }
    }

    /// Tracks only `### FILE:` blocks, leaving `### PATCH:` headers and
    /// bodies untouched — for `parse_patch_blocks`, which must keep
    /// processing those lines itself rather than have them skipped here too.
    fn file_blocks_only() -> Self {
        Self {
            closing_marker: None,
            recognize_patch_headers: false,
        }
    }

    /// Feed the next trimmed line. Returns `true` if this line lies outside
    /// any block body (a header line, a closer line, or top-level text), and
    /// `false` if it lies inside one.
    fn consume(&mut self, trimmed: &str) -> bool {
        if let Some(closer) = self.closing_marker {
            if trimmed == closer {
                self.closing_marker = None;
            }
            return false;
        }
        if trimmed.strip_prefix(FILE_MARKER).is_some() {
            self.closing_marker = Some(END_MARKER);
        } else if self.recognize_patch_headers
            && trimmed.strip_prefix(PATCH_HEADER_MARKER).is_some()
        {
            self.closing_marker = Some(REPLACE_END_MARKER);
        }
        true
    }
}

/// Extracts `SUMMARY:` and `RATIONALE:` header lines independent of any
/// `### FILE:` or `### PATCH:` block. Shared by `parse_fenced_edits` and the
/// patch transport so summary/rationale extraction never diverges between
/// them — in particular, a patch-only response (the common case once patches
/// exist) is not silently treated as carrying no summary or rationale just
/// because it has no `### FILE:` block for the old, file-block-gated logic
/// to key off of.
pub fn parse_summary_and_rationale(raw: &str) -> (String, Vec<String>) {
    let mut summary = String::new();
    let mut rationale = Vec::new();
    let mut in_rationale = false;
    let mut tracker = BlockTracker::new();

    for line in raw.split('\n') {
        let line_for_markers = line.trim_end_matches('\r');
        let trimmed = line_for_markers.trim();

        if !tracker.consume(trimmed) {
            continue;
        }

        if trimmed.strip_prefix(FILE_MARKER).is_some()
            || trimmed.strip_prefix(PATCH_HEADER_MARKER).is_some()
        {
            in_rationale = false;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
            summary = rest.trim().to_string();
            in_rationale = false;
        } else if trimmed == "RATIONALE:" {
            in_rationale = true;
        } else if in_rationale {
            if let Some(item) = trimmed.strip_prefix("- ") {
                rationale.push(item.trim().to_string());
            }
        }
    }

    (summary, rationale)
}

/// True only when a `### PATCH:` header appears outside of any `### FILE:` …
/// `### END FILE` block — i.e. as a real patch, not as text quoted inside a
/// whole file's documentation content. `PATCH_FORMAT_INSTRUCTION` is itself a
/// complete example patch block and now appears in every Mason system
/// prompt, so a model writing documentation that quotes it back is not a
/// contrived case: a naive `raw.contains("### PATCH:")` check would treat
/// that quoted example as a real patch and reject (or misparse) an otherwise
/// valid whole-file response.
pub fn has_top_level_patch_header(raw: &str) -> bool {
    let mut tracker = BlockTracker::new();

    for line in raw.split('\n') {
        let line_for_markers = line.trim_end_matches('\r');
        let trimmed = line_for_markers.trim();

        let is_top_level = tracker.consume(trimmed);
        if is_top_level && trimmed.strip_prefix(PATCH_HEADER_MARKER).is_some() {
            return true;
        }
    }

    false
}

pub fn parse_fenced_edits(raw: &str) -> Result<FencedEnvelope> {
    let (summary, rationale) = parse_summary_and_rationale(raw);
    let mut files = Vec::new();

    // This loop implements its own FILE-block open/close tracking (`current`)
    // rather than going through `BlockTracker`, because it also needs to
    // accumulate the body content and raise the specific errors below —
    // `BlockTracker` only reports in/out. It agrees with `BlockTracker` on
    // what closes a FILE block only because both compare against the same
    // `END_MARKER` constant; that constant is the entire coupling. If this
    // loop's notion of "closed" ever changes, `BlockTracker` (and therefore
    // `parse_summary_and_rationale`, `has_top_level_patch_header`, and
    // `parse_patch_blocks`'s FILE-block skipping) must change with it.
    let mut current: Option<(String, Vec<String>)> = None;

    // Split on \n but preserve original lines (including trailing \r for CRLF)
    let lines: Vec<&str> = raw.split('\n').collect();
    for (line_num, line) in lines.iter().enumerate() {
        // For marker detection, work with a version that has \r stripped from the end
        let line_for_markers = line.trim_end_matches('\r');
        let trimmed = line_for_markers.trim();

        // If we're inside a file block, accumulate content
        if let Some((path, body)) = current.as_mut() {
            // Check if this line is exactly the END_MARKER
            if trimmed == END_MARKER {
                let path = path.clone();
                let content = body.join("\n");
                files.push(FencedFile { path, content });
                current = None;
            } else if let Some(_) = trimmed.strip_prefix(FILE_MARKER) {
                // A FILE_MARKER encountered inside an open block is an error
                bail!(
                    "encountered a {FILE_MARKER} marker at line {} while the {FILE_MARKER} \
                     block for {path:?} was still open — file content must not contain a line \
                     consisting solely of '### FILE:' or '### END FILE'",
                    line_num + 1
                );
            } else {
                // Add the original line (preserving \r if present)
                body.push(line.to_string());
            }
            continue;
        }

        // We're not in a file block; check for markers
        if let Some(rest) = trimmed.strip_prefix(FILE_MARKER) {
            let path = rest.trim().to_string();
            if path.is_empty() {
                bail!("a {FILE_MARKER} block declared an empty path");
            }
            if path.contains('"') || path.len() > 200 {
                bail!("a {FILE_MARKER} block declared an implausible path: {path:?}");
            }
            current = Some((path, Vec::new()));
        } else if trimmed == END_MARKER {
            // A stray END_MARKER outside any open block is an error
            bail!(
                "found a stray {END_MARKER} at line {} not associated with any open file block — \
                 file content must not contain a line consisting solely of '### FILE:' or \
                 '### END FILE'",
                line_num + 1
            );
        }
        // SUMMARY:/RATIONALE:/rationale-item lines are metadata already
        // captured by `parse_summary_and_rationale` above; nothing to do
        // with them here.
    }

    // An unterminated block is a truncated response, not a formatting mistake.
    // Reporting it as such points the operator at the token budget rather than
    // at the model's punctuation.
    if let Some((path, _)) = current {
        bail!(
            "the {FILE_MARKER} block for {path:?} was never closed with {END_MARKER} — the \
             response was cut off. Raise MASON_EDIT_MAX_TOKENS or narrow the editable surface."
        );
    }

    if files.is_empty() {
        bail!("no {FILE_MARKER} blocks were found in the response");
    }

    Ok(FencedEnvelope {
        summary,
        rationale,
        files,
    })
}

pub const PATCH_FORMAT_INSTRUCTION: &str = "\
When changing an existing file, emit a patch rather than the whole file:

### PATCH: <relative/path>
<<<<<<< SEARCH
<text to find, copied exactly from the current file>
=======
<text to put in its place>
>>>>>>> REPLACE

The SEARCH text must appear exactly once in the file, copied character for \
character including indentation. Use a whole ### FILE: block instead when \
creating a new file.

IMPORTANT: The SEARCH and REPLACE sections must not contain lines that consist \
solely of '<<<<<<< SEARCH', '=======', or '>>>>>>> REPLACE', nor may they start \
with '### PATCH:' — the parser cannot distinguish such lines from real delimiters. \
If a file contains these lines, use a whole ### FILE: block instead of a patch.

IMPORTANT: The format cannot express a trailing newline in the REPLACE section. \
To delete a line, include an adjacent line as context in both SEARCH and REPLACE. \
For example, to delete 'line 2' from a three-line file, search for 'line 1\\nline 2' \
and replace with 'line 1' — do not search for just 'line 2' and replace with empty, \
which would leave a blank line.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchBlock {
    pub path: String,
    pub search: String,
    pub replace: String,
}

/// Exact match only, and it must be unique. A fuzzy matcher would apply more
/// patches, and would sometimes apply them in the wrong place — which is
/// unrecoverable once written. Refusing costs a retry; guessing costs the file.
pub fn apply_patch_block(original: &str, block: &PatchBlock) -> Result<String> {
    if block.search.is_empty() {
        bail!("patch for {} has an empty SEARCH section", block.path);
    }
    let occurrences = original.matches(block.search.as_str()).count();
    match occurrences {
        0 => bail!(
            "patch for {} did not match: the SEARCH text is not present in the file",
            block.path
        ),
        1 => Ok(original.replacen(block.search.as_str(), &block.replace, 1)),
        n => bail!(
            "patch for {} is ambiguous: the SEARCH text matched {n} times, so the target is \
             unclear. Include more surrounding context to make it unique.",
            block.path
        ),
    }
}

pub fn parse_patch_blocks(raw: &str) -> Result<Vec<PatchBlock>> {
    const PATCH_MARKER: &str = "### PATCH:";
    const SEARCH_START: &str = "<<<<<<< SEARCH";
    const DIVIDER: &str = "=======";
    const REPLACE_END: &str = ">>>>>>> REPLACE";

    let mut blocks = Vec::new();
    let mut path: Option<String> = None;
    let mut search: Vec<String> = Vec::new();
    let mut replace: Vec<String> = Vec::new();
    let mut section = 0u8; // 0 outside, 1 in SEARCH, 2 in REPLACE

    // This state machine is a flat scan over the whole response with no
    // awareness of `### FILE:` boundaries on its own — patch markers inside
    // a file block's body are content, not structure, so they must never
    // reach the checks below. A `### FILE:` block that documents or quotes
    // PATCH_FORMAT_INSTRUCTION (syntactically valid patch grammar, sitting
    // right there as file content) would otherwise be parsed as a second,
    // bogus patch and reject the entire response, permanently, alongside a
    // real top-level patch in the same reply.
    let mut file_tracker = BlockTracker::file_blocks_only();

    for (line_idx, line) in raw.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim_end();
        let marker_trim = trimmed.trim();

        if !file_tracker.consume(marker_trim) {
            continue;
        }

        if let Some(rest) = marker_trim.strip_prefix(PATCH_MARKER) {
            // Guard: no new patch while one is open (section != 0)
            if section != 0 {
                let prev_path = path.as_ref().map(|p| p.as_str()).unwrap_or("(unknown)");
                let new_path = rest.trim();
                bail!(
                    "line {line_num}: a new patch header appeared while the block for \
                     {prev_path:?} was still open — found {PATCH_MARKER} {new_path:?}"
                );
            }
            path = Some(rest.trim().to_string());
            search.clear();
            replace.clear();
            section = 0;
        } else if marker_trim == SEARCH_START {
            // Guard: SEARCH marker only valid outside a block (section == 0)
            if section != 0 {
                bail!(
                    "line {line_num}: found {SEARCH_START} inside an open block (section {section}), \
                     expected only at the start of a new block"
                );
            }
            section = 1;
        } else if marker_trim == DIVIDER {
            // Guard: divider only valid in SEARCH section (section == 1)
            if section == 1 {
                section = 2;
            } else if section == 2 {
                bail!(
                    "line {line_num}: found a second {DIVIDER} in one patch block, \
                     each block has exactly one divider"
                );
            } else {
                bail!(
                    "line {line_num}: found {DIVIDER} outside of SEARCH section (section {section}), \
                     expected only after {SEARCH_START}"
                );
            }
        } else if marker_trim == REPLACE_END {
            // Guard: terminator only valid in REPLACE section (section == 2)
            if section == 2 {
                let Some(current_path) = path.clone() else {
                    bail!("line {line_num}: found {REPLACE_END} without a preceding {PATCH_MARKER} line");
                };
                blocks.push(PatchBlock {
                    path: current_path,
                    search: search.join("\n"),
                    replace: replace.join("\n"),
                });
                path = None;
                search.clear();
                replace.clear();
                section = 0;
            } else if section == 1 {
                bail!(
                    "line {line_num}: found {REPLACE_END} while still in SEARCH section, \
                     expected {DIVIDER} before {REPLACE_END}"
                );
            } else {
                bail!(
                    "line {line_num}: found {REPLACE_END} outside of any patch block (section {section}), \
                     no open block to close"
                );
            }
        } else if section == 1 {
            search.push(line.to_string());
        } else if section == 2 {
            replace.push(line.to_string());
        }
    }

    if section != 0 {
        bail!("a patch block was never closed with {REPLACE_END} — the response was cut off");
    }
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenced_envelope_passes_source_code_through_verbatim() {
        let raw = r#"SUMMARY: Add the bonus room
RATIONALE:
- followed the existing G.rooms shape
- registered the room in main.js

### FILE: js/bonus.js
G.rooms.bonus = {
  id: 'bonus',
  verbs: { lookat: "A dusty attic", open: "It creaks" },
  note: "quotes \" and backslashes \\ survive"
};
### END FILE

### FILE: README.md
The bonus room ("Dad's Workshop") is optional.
### END FILE
"#;

        let envelope = parse_fenced_edits(raw).expect("must parse");

        assert_eq!(envelope.summary, "Add the bonus room");
        assert_eq!(envelope.rationale.len(), 2);
        assert_eq!(envelope.files.len(), 2);
        assert_eq!(envelope.files[0].path, "js/bonus.js");
        assert!(envelope.files[0]
            .content
            .contains(r#"lookat: "A dusty attic""#));
        assert!(envelope.files[0]
            .content
            .contains(r#"backslashes \\ survive"#));
        assert!(envelope.files[1].content.contains(r#"("Dad's Workshop")"#));
        assert!(
            !envelope.files[0].content.contains("### END FILE"),
            "the terminator must not leak into content"
        );
    }

    #[test]
    fn fenced_envelope_reports_an_unterminated_file_as_truncation() {
        let raw = "SUMMARY: x\n\n### FILE: js/a.js\nG.rooms.a = {};\n";
        let error = parse_fenced_edits(raw).expect_err("unterminated block must fail");
        assert!(
            format!("{error:#}").contains("never closed"),
            "an unterminated block means a cut-off response, got: {error:#}"
        );
    }

    #[test]
    fn fenced_envelope_rejects_a_path_that_is_not_a_path() {
        let raw = "SUMMARY: x\n\n### FILE: \n### END FILE\n";
        assert!(
            parse_fenced_edits(raw).is_err(),
            "empty path must be refused"
        );
    }

    #[test]
    fn fenced_envelope_rejects_content_containing_a_terminator_line() {
        // Finding 1: a ### END FILE on a line by itself inside content should error, not truncate
        let raw = "SUMMARY: Fix\n\n### FILE: a.txt\nbefore\n### END FILE\nafter\n### END FILE\n";
        let error = parse_fenced_edits(raw).expect_err("content with terminator must fail");
        let error_msg = format!("{error:#}");
        assert!(
            error_msg.contains("stray") && error_msg.contains("not associated"),
            "should report stray terminator, got: {error_msg}"
        );
    }

    #[test]
    fn fenced_envelope_rejects_nested_file_marker() {
        // Finding 2: a ### FILE: inside an open block should error, not be swallowed
        let raw = "SUMMARY: Fix\n\n### FILE: a.txt\nfirst\n### FILE: b.txt\nsecond\n### END FILE\n";
        let error = parse_fenced_edits(raw).expect_err("nested file marker must fail");
        let error_msg = format!("{error:#}");
        assert!(
            error_msg.contains("while the") && error_msg.contains("still open"),
            "should report nested marker, got: {error_msg}"
        );
    }

    #[test]
    fn fenced_envelope_accepts_indented_terminator() {
        // Finding 3: an indented ### END FILE should be recognized as a terminator (symmetric trim)
        let raw = "SUMMARY: Fix\n\n### FILE: a.txt\ncontent\n  ### END FILE\n";
        let envelope = parse_fenced_edits(raw).expect("indented terminator must parse");
        assert_eq!(envelope.files.len(), 1);
        assert_eq!(envelope.files[0].path, "a.txt");
        assert_eq!(envelope.files[0].content, "content");
    }

    #[test]
    fn fenced_envelope_preserves_crlf_line_endings() {
        // Finding 4: CRLF line endings should round-trip unchanged (not be stripped to LF)
        // When split on '\n', CRLF becomes visible as \r in each line, verifying \r is preserved
        let raw = "SUMMARY: Fix\r\n\r\n### FILE: a.txt\r\nline1\r\nline2\r\n### END FILE\r\n";
        let envelope = parse_fenced_edits(raw).expect("CRLF must parse");
        assert_eq!(envelope.files.len(), 1);
        // The content preserves \r from CRLF line endings (not stripped by .lines())
        // line1\r\nline2\r represents two lines with CRLF on the first, and CR remaining on the
        // second from the split
        assert!(
            envelope.files[0].content.contains("\r"),
            "CRLF must be preserved, not stripped to LF"
        );
        assert_eq!(envelope.files[0].content, "line1\r\nline2\r");
    }

    #[test]
    fn fenced_envelope_rejects_prose_starting_with_file_marker() {
        // Residual finding: content line starting with "### FILE:" but containing more
        // (like "### FILE: this is prose about files") must error with nested marker message
        // even though it's not an exact marker match, because the parser cannot distinguish it
        let raw = "SUMMARY: Docs\n\n### FILE: README.md\n\
                   The envelope format is described in task-2-brief.md.\n\
                   ### FILE: this is prose about files, not a marker\n\
                   But it starts with the marker phrase.\n\
                   ### END FILE\n";
        let error = parse_fenced_edits(raw).expect_err("prose starting with marker must fail");
        let error_msg = format!("{error:#}");
        assert!(
            error_msg.contains("while the") && error_msg.contains("still open"),
            "should report nested marker, got: {error_msg}"
        );
    }

    #[test]
    fn patch_block_replaces_an_exact_region() {
        let original = "line a\nline b\nline c\n";
        let block = PatchBlock {
            path: "js/a.js".to_string(),
            search: "line b".to_string(),
            replace: "line B1\nline B2".to_string(),
        };

        let patched = apply_patch_block(original, &block).expect("must apply");

        assert_eq!(patched, "line a\nline B1\nline B2\nline c\n");
    }

    #[test]
    fn patch_block_refuses_when_the_search_text_is_absent() {
        let block = PatchBlock {
            path: "js/a.js".to_string(),
            search: "nowhere".to_string(),
            replace: "x".to_string(),
        };
        let error = apply_patch_block("line a\n", &block).expect_err("must refuse");
        assert!(format!("{error:#}").contains("did not match"));
    }

    #[test]
    fn patch_block_refuses_an_ambiguous_match() {
        let block = PatchBlock {
            path: "js/a.js".to_string(),
            search: "dup".to_string(),
            replace: "x".to_string(),
        };
        let error = apply_patch_block("dup\ndup\n", &block).expect_err("must refuse");
        assert!(format!("{error:#}").contains("matched 2 times"));
    }

    #[test]
    fn patch_blocks_parse_from_the_wire_format() {
        let raw = "### PATCH: js/a.js\n<<<<<<< SEARCH\nline b\n=======\nline B\n>>>>>>> REPLACE\n";
        let blocks = parse_patch_blocks(raw).expect("must parse");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].path, "js/a.js");
        assert_eq!(blocks[0].search, "line b");
        assert_eq!(blocks[0].replace, "line B");
    }

    #[test]
    fn has_top_level_patch_header_detects_a_real_patch() {
        let raw =
            "SUMMARY: x\n\n### PATCH: js/a.js\n<<<<<<< SEARCH\na\n=======\nb\n>>>>>>> REPLACE\n";
        assert!(has_top_level_patch_header(raw));
    }

    #[test]
    fn has_top_level_patch_header_ignores_a_patch_quoted_inside_a_file_block() {
        // The exact failure mode this function exists to prevent: PATCH_FORMAT_INSTRUCTION
        // is a complete example patch block, and it now appears in every Mason
        // system prompt, so a whole-file response documenting or quoting it
        // back must not be mistaken for a real patch.
        let raw = format!(
            "SUMMARY: docs\n\n### FILE: docs/PATCHES.md\n{}\n### END FILE\n",
            PATCH_FORMAT_INSTRUCTION
        );
        assert!(
            !has_top_level_patch_header(&raw),
            "a ### PATCH: line quoted inside a ### FILE: block must not count as top-level"
        );
    }

    #[test]
    fn has_top_level_patch_header_is_false_with_no_patch_marker_at_all() {
        let raw = "SUMMARY: x\n\n### FILE: a.txt\ncontent\n### END FILE\n";
        assert!(!has_top_level_patch_header(raw));
    }

    #[test]
    fn has_top_level_patch_header_still_finds_a_real_patch_after_a_file_block() {
        let raw = "SUMMARY: x\n\n### FILE: a.txt\ncontent\n### END FILE\n\n### PATCH: b.js\n<<<<<<< SEARCH\na\n=======\nb\n>>>>>>> REPLACE\n";
        assert!(has_top_level_patch_header(raw));
    }

    #[test]
    fn parse_patch_blocks_rejects_duplicate_divider_in_replace() {
        // Defect 1: a second ======= inside REPLACE section should error
        let raw = "### PATCH: js/a.js\n<<<<<<< SEARCH\nsearch\n=======\nreplace1\n=======\nmore\n>>>>>>> REPLACE\n";
        let error = parse_patch_blocks(raw).expect_err("must reject second divider");
        let msg = format!("{error:#}");
        assert!(msg.contains("=======") && msg.contains("second"));
    }

    #[test]
    fn parse_patch_blocks_rejects_stray_terminator() {
        // Defect 2: >>>>>>> REPLACE appearing after a block closes is a stray terminator
        let raw = "### PATCH: js/a.js\n<<<<<<< SEARCH\nsearch\n=======\nreplace\n>>>>>>> REPLACE\ntrailing\n>>>>>>> REPLACE\n";
        let error = parse_patch_blocks(raw).expect_err("must reject stray terminator");
        let msg = format!("{error:#}");
        assert!(msg.contains(">>>>>>> REPLACE") || msg.contains("no open"));
    }

    #[test]
    fn parse_patch_blocks_rejects_search_marker_in_replace_section() {
        // Defect 3: <<<<<<< SEARCH inside REPLACE section should error
        let raw = "### PATCH: js/a.js\n<<<<<<< SEARCH\nsearch\n=======\nline1\n<<<<<<< SEARCH\nline2\n>>>>>>> REPLACE\n";
        let error =
            parse_patch_blocks(raw).expect_err("must reject SEARCH marker in REPLACE section");
        let msg = format!("{error:#}");
        assert!(msg.contains("<<<<<<< SEARCH"));
    }

    #[test]
    fn parse_patch_blocks_rejects_overlapping_patch_headers() {
        // Defect 4: ### PATCH: appearing before previous block closes silently discards it
        let raw = "### PATCH: a.js\n<<<<<<< SEARCH\nsearch a\n=======\nreplace a\n### PATCH: b.js\n<<<<<<< SEARCH\nsearch b\n=======\nreplace b\n>>>>>>> REPLACE\n";
        let error = parse_patch_blocks(raw).expect_err("must reject overlapping blocks");
        let msg = format!("{error:#}");
        assert!(msg.contains("a.js") || msg.contains("while the block"));
    }

    #[test]
    fn patch_format_cannot_express_trailing_newline() {
        // Defect 5: The format cannot express trailing newlines; deleting a line without context leaves a blank
        let block = PatchBlock {
            path: "test.txt".to_string(),
            search: "line 2".to_string(),
            replace: "".to_string(),
        };
        let original = "line 1\nline 2\nline 3\n";
        let result = apply_patch_block(original, &block).expect("must apply");
        // Deleting "line 2" leaves a blank line because the search doesn't include the newline
        assert_eq!(result, "line 1\n\nline 3\n");
    }
}
