const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("AWS Access Key", "AKIA"),
    ("AWS Secret Key", "aws_secret_access_key"),
    ("GitHub Token (classic)", "ghp_"),
    ("GitHub Token (fine-grained)", "github_pat_"),
    ("GitHub OAuth", "gho_"),
    ("GitHub App User-to-Server", "ghu_"),
    ("GitHub App Server-to-Server", "ghs_"),
    ("GitHub App Refresh", "ghr_"),
    ("GitLab Token", "glpat-"),
    ("Slack Bot Token", "xoxb-"),
    ("Slack User Token", "xoxp-"),
    ("Slack Webhook", "hooks.slack.com/services/"),
    ("Stripe Secret Key", "sk_live_"),
    ("Stripe Restricted Key", "rk_live_"),
    ("Google API Key", "AIza"),
    ("Anthropic API Key", "sk-ant-"),
    ("OpenAI API Key", "sk-proj-"),
    ("Twilio Auth Token", "twilio_auth_token"),
    ("SendGrid API Key", "SG."),
    ("Mailgun API Key", "key-"),
    ("Heroku API Key", "heroku_api_key"),
    ("Private Key Header", "-----BEGIN RSA PRIVATE KEY-----"),
    ("Private Key Header (EC)", "-----BEGIN EC PRIVATE KEY-----"),
    ("Private Key Header (generic)", "-----BEGIN PRIVATE KEY-----"),
    ("Generic password field", "password="),
    ("Generic secret field", "secret="),
    ("Bearer token literal", "Bearer eyJ"),
    ("JWT token (raw)", "eyJhbGciOi"),
    ("Database URL with password", "postgresql://"),
    ("Database URL with password (mysql)", "mysql://"),
    ("MongoDB connection string", "mongodb+srv://"),
    ("Redis URL with auth", "redis://"),
];

fn collect_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_strings(v, out);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_strings(v, out);
            }
        }
        _ => {}
    }
}

pub(super) fn detect_secrets(
    tool_input: Option<&serde_json::Value>, // JSON: required by HookEventPayload contract
) -> Option<(String, String)> {
    let input = tool_input?;

    let mut strings = Vec::new();
    collect_strings(input, &mut strings);

    for s in &strings {
        for &(pattern_name, prefix) in SECRET_PATTERNS {
            if s.contains(prefix) {
                let match_start = s.find(prefix).unwrap_or(0);
                let snippet_end = (match_start + 12).min(s.len());
                let redacted = format!("{}...[REDACTED]", &s[match_start..snippet_end]);
                return Some((pattern_name.to_string(), redacted));
            }
        }
    }

    None
}
