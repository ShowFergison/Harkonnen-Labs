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
block per file, and nothing after the final ### END FILE. Important: no line \
inside any file's content may consist solely of '### FILE:' or '### END FILE' — \
the parser uses those to delimit blocks and cannot distinguish them from content.";

const FILE_MARKER: &str = "### FILE:";
const END_MARKER: &str = "### END FILE";

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

pub fn parse_fenced_edits(raw: &str) -> Result<FencedEnvelope> {
    let mut summary = String::new();
    let mut rationale = Vec::new();
    let mut files = Vec::new();

    let mut current: Option<(String, Vec<String>)> = None;
    let mut in_rationale = false;

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

        // We're not in a file block; check for markers or metadata
        if let Some(rest) = trimmed.strip_prefix(FILE_MARKER) {
            let path = rest.trim().to_string();
            if path.is_empty() {
                bail!("a {FILE_MARKER} block declared an empty path");
            }
            if path.contains('"') || path.len() > 200 {
                bail!("a {FILE_MARKER} block declared an implausible path: {path:?}");
            }
            in_rationale = false;
            current = Some((path, Vec::new()));
        } else if trimmed == END_MARKER {
            // A stray END_MARKER outside any open block is an error
            bail!(
                "found a stray {END_MARKER} at line {} not associated with any open file block — \
                 file content must not contain a line consisting solely of '### FILE:' or \
                 '### END FILE'",
                line_num + 1
            );
        } else if let Some(rest) = trimmed.strip_prefix("SUMMARY:") {
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
}
