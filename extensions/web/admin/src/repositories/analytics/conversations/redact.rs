//! Credential redactor for transcript bodies heading to the DOM.

// Why: Defense-in-depth text redactor for prompts/responses heading to the DOM.
//
// Catches the common high-entropy / well-prefixed credential shapes; not a
// substitute for the structural `secret_scan` policy run at webhook time.
//
// The count accompanying the redacted text is how many replacements were
// made, so a caller can surface that redaction occurred at all.
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
    redact_ssn(&out, count)
}

// Why: a second pass for the one PII shape worth masking in a transcript view
// — the canonical AAA-GG-SSSS SSN, digit-bounded so ids and hashes pass
// through. Last four kept, the convention SSNs are displayed with everywhere.
fn redact_ssn(input: &str, mut count: u32) -> (String, u32) {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut idx = 0usize;
    while idx < bytes.len() {
        if idx + 11 <= bytes.len() && is_ssn_at(bytes, idx) {
            out.push_str("***-**-");
            out.push_str(&input[idx + 7..idx + 11]);
            count = count.saturating_add(1);
            idx += 11;
        } else {
            let ch = input[idx..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&input[idx..idx + ch]);
            idx += ch;
        }
    }
    (out, count)
}

fn is_ssn_at(bytes: &[u8], i: usize) -> bool {
    let w = &bytes[i..i + 11];
    let shape = w[..3].iter().all(u8::is_ascii_digit)
        && w[3] == b'-'
        && w[4..6].iter().all(u8::is_ascii_digit)
        && w[6] == b'-'
        && w[7..].iter().all(u8::is_ascii_digit);
    if !shape {
        return false;
    }
    let before = i.checked_sub(1).map(|j| bytes[j]);
    let after = bytes.get(i + 11).copied();
    let bounded = !before.is_some_and(|b| b.is_ascii_digit() || b == b'-')
        && !after.is_some_and(|b| b.is_ascii_digit() || b == b'-');
    bounded && w[0] != b'9' && &w[..3] != b"000" && &w[..3] != b"666"
}
