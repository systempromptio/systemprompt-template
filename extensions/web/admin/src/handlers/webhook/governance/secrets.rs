//! Built-in plaintext secret-pattern registry.
//!
//! Each pattern has three pieces:
//! - `id`: a stable kebab-case identifier — the persistent referent written
//!   into `governance_decisions.evaluated_rules.pattern_id` and used by
//!   dashboards / SQL aggregations. NEVER renamed.
//! - `name`: human-readable label rendered in deny messages.
//! - `prefix`: substring that triggers the match.

#[derive(Debug, Clone, Copy)]
pub(super) struct SecretPattern {
    pub id: &'static str,
    pub name: &'static str,
    pub prefix: &'static str,
}

pub(super) const SECRET_PATTERNS: &[SecretPattern] = &[
    SecretPattern {
        id: "aws-access-key",
        name: "AWS Access Key",
        prefix: "AKIA",
    },
    SecretPattern {
        id: "aws-secret-key",
        name: "AWS Secret Key",
        prefix: "aws_secret_access_key",
    },
    SecretPattern {
        id: "github-token-classic",
        name: "GitHub Token (classic)",
        prefix: "ghp_",
    },
    SecretPattern {
        id: "github-token-fine-grained",
        name: "GitHub Token (fine-grained)",
        prefix: "github_pat_",
    },
    SecretPattern {
        id: "github-oauth",
        name: "GitHub OAuth",
        prefix: "gho_",
    },
    SecretPattern {
        id: "github-app-user-to-server",
        name: "GitHub App User-to-Server",
        prefix: "ghu_",
    },
    SecretPattern {
        id: "github-app-server-to-server",
        name: "GitHub App Server-to-Server",
        prefix: "ghs_",
    },
    SecretPattern {
        id: "github-app-refresh",
        name: "GitHub App Refresh",
        prefix: "ghr_",
    },
    SecretPattern {
        id: "gitlab-token",
        name: "GitLab Token",
        prefix: "glpat-",
    },
    SecretPattern {
        id: "slack-bot-token",
        name: "Slack Bot Token",
        prefix: "xoxb-",
    },
    SecretPattern {
        id: "slack-user-token",
        name: "Slack User Token",
        prefix: "xoxp-",
    },
    SecretPattern {
        id: "slack-webhook",
        name: "Slack Webhook",
        prefix: "hooks.slack.com/services/",
    },
    SecretPattern {
        id: "stripe-secret-key",
        name: "Stripe Secret Key",
        prefix: "sk_live_",
    },
    SecretPattern {
        id: "stripe-restricted-key",
        name: "Stripe Restricted Key",
        prefix: "rk_live_",
    },
    SecretPattern {
        id: "google-api-key",
        name: "Google API Key",
        prefix: "AIza",
    },
    SecretPattern {
        id: "anthropic-api-key",
        name: "Anthropic API Key",
        prefix: "sk-ant-",
    },
    SecretPattern {
        id: "openai-api-key",
        name: "OpenAI API Key",
        prefix: "sk-proj-",
    },
    SecretPattern {
        id: "twilio-auth-token",
        name: "Twilio Auth Token",
        prefix: "twilio_auth_token",
    },
    SecretPattern {
        id: "sendgrid-api-key",
        name: "SendGrid API Key",
        prefix: "SG.",
    },
    SecretPattern {
        id: "mailgun-api-key",
        name: "Mailgun API Key",
        prefix: "key-",
    },
    SecretPattern {
        id: "heroku-api-key",
        name: "Heroku API Key",
        prefix: "heroku_api_key",
    },
    SecretPattern {
        id: "pem-private-key-rsa",
        name: "RSA Private Key",
        prefix: "-----BEGIN RSA PRIVATE KEY-----",
    },
    SecretPattern {
        id: "pem-private-key-ec",
        name: "EC Private Key",
        prefix: "-----BEGIN EC PRIVATE KEY-----",
    },
    SecretPattern {
        id: "pem-private-key",
        name: "PEM Private Key",
        prefix: "-----BEGIN PRIVATE KEY-----",
    },
    SecretPattern {
        id: "generic-password-field",
        name: "Generic password field",
        prefix: "password=",
    },
    SecretPattern {
        id: "generic-secret-field",
        name: "Generic secret field",
        prefix: "secret=",
    },
    SecretPattern {
        id: "bearer-token-jwt",
        name: "Bearer token (JWT)",
        prefix: "Bearer eyJ",
    },
    SecretPattern {
        id: "jwt-raw",
        name: "JWT token (raw)",
        prefix: "eyJhbGciOi",
    },
    SecretPattern {
        id: "postgres-url-with-password",
        name: "Postgres URL with password",
        prefix: "postgresql://",
    },
    SecretPattern {
        id: "mysql-url-with-password",
        name: "MySQL URL with password",
        prefix: "mysql://",
    },
    SecretPattern {
        id: "mongodb-srv-url",
        name: "MongoDB connection string",
        prefix: "mongodb+srv://",
    },
    SecretPattern {
        id: "redis-url-with-auth",
        name: "Redis URL with auth",
        prefix: "redis://",
    },
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

/// Scan a flat string for the first matching secret pattern, returning a
/// redacted excerpt. Shares [`SECRET_PATTERNS`] with the governance webhook so
/// the gateway safety scanner and the tool-use governor flag the same
/// credentials.
pub fn scan_str_for_secret(text: &str) -> Option<String> {
    for pattern in SECRET_PATTERNS {
        if let Some(match_start) = text.find(pattern.prefix) {
            let snippet_end = (match_start + 12).min(text.len());
            return Some(format!("{}...[REDACTED]", &text[match_start..snippet_end]));
        }
    }
    None
}

pub(super) fn detect_secrets(
    tool_input: Option<&serde_json::Value>,
) -> Option<(&'static SecretPattern, String)> {
    let input = tool_input?;

    let mut strings = Vec::new();
    collect_strings(input, &mut strings);

    for s in &strings {
        for pattern in SECRET_PATTERNS {
            if let Some(match_start) = s.find(pattern.prefix) {
                let snippet_end = (match_start + 12).min(s.len());
                let redacted = format!("{}...[REDACTED]", &s[match_start..snippet_end]);
                return Some((pattern, redacted));
            }
        }
    }

    None
}
