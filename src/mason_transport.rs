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
block per file, and nothing after the final ### END FILE.";

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

    for line in raw.lines() {
        if let Some((path, body)) = current.as_mut() {
            if line.trim_end() == END_MARKER {
                let path = path.clone();
                let content = body.join("\n");
                files.push(FencedFile { path, content });
                current = None;
            } else {
                body.push(line.to_string());
            }
            continue;
        }

        let trimmed = line.trim();
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
}
