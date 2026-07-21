//! Transcript JSONB parsing — turns the stored `session_transcripts.transcript`
//! array into normalised `TranscriptTurn`s, extracts tool calls and body text,
//! and attaches the per-session governance decision.


/// Pull a textual representation of a transcript entry's body.
/// Accepts plain strings, Anthropic-style content arrays, or `text` fields.
pub(super) fn extract_content_text(entry: &serde_json::Value) -> String {
    if let Some(s) = entry.get("content").and_then(|v| v.as_str()) {
        return s.to_owned();
    }
    if let Some(arr) = entry.get("content").and_then(|v| v.as_array()) {
        let mut out = String::new();
        for block in arr {
            if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(t);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    if let Some(s) = entry.get("text").and_then(|v| v.as_str()) {
        return s.to_owned();
    }
    String::new()
}
