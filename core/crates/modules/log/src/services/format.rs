//! Log Format Utilities
//!
//! Standard format: `{action} | {key}={value}, {key}={value}`
//!
//! Rules:
//! - No emojis
//! - Past tense for completions, present for starts
//! - Pipe separator distinguishes message from metadata
//! - `snake_case` keys

pub fn format_log(message: &str, pairs: &[(&str, &str)]) -> String {
    if pairs.is_empty() {
        message.to_string()
    } else {
        let kv = pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{message} | {kv}")
    }
}

pub fn format_log_owned(message: &str, pairs: &[(&str, String)]) -> String {
    if pairs.is_empty() {
        message.to_string()
    } else {
        let kv = pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{message} | {kv}")
    }
}
