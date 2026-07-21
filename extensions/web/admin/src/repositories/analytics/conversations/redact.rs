//! Credential redactor for transcript bodies heading to the DOM.

/// Defense-in-depth text redactor for prompts/responses heading to the DOM.
///
/// Catches the common high-entropy / well-prefixed credential shapes; not a
/// substitute for the structural `secret_scan` policy run at webhook time.
///
/// The count accompanying the redacted text is how many replacements were
/// made, so a caller can surface that redaction occurred at all.
pub fn redact_text(input: &str) -> (String, u32) {
    const PREFIX_PATTERNS: &[(&str, &str)] = &[
        ("AKIA", "aws_access_key"),
        ("ASIA", "aws_session_key"),
        ("ghp_", "github_token"),
        ("github_pat_", "github_token"),
        ("gho_", "github_oauth"),
        ("ghu_", "github_user_token"),
        ("ghs_", "github_server_token"),
        ("ghr_", "github_refresh"),
        ("glpat-", "gitlab_token"),
        ("xoxb-", "slack_bot_token"),
        ("xoxp-", "slack_user_token"),
        ("sk-ant-", "anthropic_api_key"),
        ("sk-proj-", "openai_api_key"),
        ("sk_live_", "stripe_secret_key"),
        ("rk_live_", "stripe_restricted_key"),
        ("AIza", "google_api_key"),
        ("SG.", "sendgrid_api_key"),
    ];

    let mut out = String::with_capacity(input.len());
    let mut count: u32 = 0;
    let mut idx = 0usize;
    let bytes = input.as_bytes();
    while idx < bytes.len() {
        let mut hit: Option<(usize, &str)> = None;
        for &(prefix, label) in PREFIX_PATTERNS {
            if input[idx..].starts_with(prefix) {
                hit = Some((prefix.len(), label));
                break;
            }
        }
        if let Some((prefix_len, label)) = hit {
            let mut end = idx + prefix_len;
            while end < bytes.len() {
                let b = bytes[end];
                if b.is_ascii_whitespace() || b == b'"' || b == b'\'' || b == b',' || b == b')' {
                    break;
                }
                end += 1;
            }
            out.push_str(&format!("[REDACTED:{label}]"));
            count = count.saturating_add(1);
            idx = end;
        } else {
            let ch = input[idx..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&input[idx..idx + ch]);
            idx += ch;
        }
    }
    (out, count)
}
