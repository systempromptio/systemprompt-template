//! Parsing of the gateway's flattened tool-use markers.
//!
//! An assistant message is stored as text with each tool use flattened to
//! `[tool_use:NAME {json}]`, several per message when the model called
//! several tools. The JSON may span lines and nest brackets, so the marker is
//! walked with a string-aware depth counter rather than a regex; an
//! unterminated marker is left in the text as it is.

const MARKER_OPEN: &str = "[tool_use:";
const REMINDER_OPEN: &str = "<system-reminder>";
const REMINDER_CLOSE: &str = "</system-reminder>";

// Why: Claude Code appends its own housekeeping to the person's prompt inside
// `<system-reminder>` blocks. They are not what was typed, so a prompt is shown
// without them; an unterminated block is left as it is.
// Why: the stored text keeps whatever indentation the client's content parts
// carried, which renders as a ragged left edge under `pre-wrap`; the words
// matter, the indentation does not.
#[must_use]
pub fn tidy_lines(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut blank_run = 0usize;
    for line in body.lines() {
        let line = line.trim_start();
        if line.is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_owned()
}

#[must_use]
pub fn strip_system_reminders(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find(REMINDER_OPEN) {
        let Some(end) = rest[start..].find(REMINDER_CLOSE) else {
            break;
        };
        out.push_str(&rest[..start]);
        rest = &rest[start + end + REMINDER_CLOSE.len()..];
    }
    out.push_str(rest);
    out.trim().to_owned()
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolUseMarker {
    pub name: String,
    pub input_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAssistant {
    pub text: String,
    pub tool_uses: Vec<ToolUseMarker>,
}

#[must_use]
pub fn parse_assistant(body: &str) -> ParsedAssistant {
    let mut text = String::new();
    let mut tool_uses = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find(MARKER_OPEN) {
        let after = &rest[start + MARKER_OPEN.len()..];
        if let Some((marker, consumed)) = parse_marker(after) {
            text.push_str(&rest[..start]);
            tool_uses.push(marker);
            rest = &after[consumed..];
        } else {
            text.push_str(&rest[..start + MARKER_OPEN.len()]);
            rest = after;
        }
    }
    text.push_str(rest);
    ParsedAssistant {
        text: text.trim().to_owned(),
        tool_uses,
    }
}

// Why: `after` starts right past `[tool_use:`; returns the marker and how many
// bytes of `after` it occupied, including the closing `]`.
fn parse_marker(after: &str) -> Option<(ToolUseMarker, usize)> {
    let name_end = after.find(|c: char| c.is_whitespace() || c == ']')?;
    let name = after[..name_end].trim();
    if name.is_empty() {
        return None;
    }
    let mut pos = name_end;
    pos += after[pos..].len() - after[pos..].trim_start().len();
    let input_json = match after[pos..].chars().next() {
        Some(']') => {
            return Some((
                ToolUseMarker {
                    name: name.to_owned(),
                    input_json: "{}".to_owned(),
                },
                pos + 1,
            ));
        },
        Some('{' | '[') => {
            let len = json_len(&after[pos..])?;
            let json = &after[pos..pos + len];
            pos += len;
            json.to_owned()
        },
        _ => return None,
    };
    pos += after[pos..].len() - after[pos..].trim_start().len();
    if !after[pos..].starts_with(']') {
        return None;
    }
    Some((
        ToolUseMarker {
            name: name.to_owned(),
            input_json,
        },
        pos + 1,
    ))
}

// Why: byte length of the JSON value at the start of `s`, honouring strings
// and escapes so a `]` inside a string argument does not end the marker.
fn json_len(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {},
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i + c.len_utf8());
                }
            },
            _ => {},
        }
    }
    None
}

// JSON: the marker's arguments are per-tool shaped and only ever re-indented.
#[must_use]
pub(super) fn pretty_json_text(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| raw.to_owned())
}
