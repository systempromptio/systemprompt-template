//! The policy editor's form, parsed into a spec.
//!
//! A plain HTML form with no JavaScript in the path: fields arrive as
//! repeated `(name, value)` pairs, windows as `w<i>_<field>`, list fields
//! (scanners, block categories) as one pair per checked box. Everything
//! here is a pure function of those pairs so the parsing is tested without
//! a request. The result is validated the way core validates the file, so
//! a spec that saves is a spec the gateway will read.

use systemprompt::ai::{
    GatewayPolicyConfig, GatewayPolicyEntry, GatewayPolicySpec, HeuristicConfig, QuotaMode,
    QuotaWindow, SafetyConfig, SafetyHistoryMode, SafetyMode,
};

pub use super::form_error::FormError;
use super::month_window::MONTH_WINDOW_SECONDS;

// Why: The bound on window rows the form renders and the parser reads.
pub const MAX_WINDOWS: usize = 8;

// Why: The subject kinds a window may name: core's `user` plus every dimension
// this extension registers a quota subject provider for.
pub const SUBJECTS: [&str; 6] = [
    "user",
    "group",
    "project",
    "role",
    "connector",
    "organization",
];

// Why: The scanners the gateway's registry resolves on this instance.
pub const SCANNERS: [&str; 3] = ["heuristic", "secrets", "pii_extended"];

// Why: Every finding category a scanner above can produce.
pub const CATEGORIES: [&str; 6] = [
    "jailbreak",
    "secret",
    "pii_ssn",
    "pii_credit_card",
    "pii_email",
    "pii_phone",
];

#[derive(Debug, Clone)]
pub struct ParsedPolicy {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub spec: GatewayPolicySpec,
}

fn first<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.trim())
        .filter(|v| !v.is_empty())
}

fn all(fields: &[(String, String)], name: &str) -> Vec<String> {
    fields
        .iter()
        .filter(|(k, _)| k == name)
        .map(|(_, v)| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .collect()
}

fn optional_i64(fields: &[(String, String)], name: &str) -> Result<Option<i64>, FormError> {
    first(fields, name)
        .map(|v| {
            v.replace([',', '_'], "")
                .parse::<i64>()
                .map_err(|source| FormError::NotWhole {
                    field: name.to_owned(),
                    source,
                })
                .and_then(|n| {
                    if n < 0 {
                        Err(FormError::Negative {
                            field: name.to_owned(),
                        })
                    } else {
                        Ok(n)
                    }
                })
        })
        .transpose()
}

// Why: dollars in the form, microdollars in the spec — a ceiling is written
// as "$200", never as 200000000.
fn optional_dollars(fields: &[(String, String)], name: &str) -> Result<Option<i64>, FormError> {
    first(fields, name)
        .map(|v| {
            v.trim_start_matches('$')
                .replace(',', "")
                .parse::<f64>()
                .map_err(|source| FormError::NotDollars {
                    field: name.to_owned(),
                    source,
                })
                .and_then(|d| {
                    if d < 0.0 {
                        Err(FormError::Negative {
                            field: name.to_owned(),
                        })
                    } else {
                        Ok((d * 1_000_000.0).round() as i64)
                    }
                })
        })
        .transpose()
}

fn window_seconds(raw: &str, index: usize) -> Result<i32, FormError> {
    match raw {
        "hour" => Ok(3_600),
        "day" => Ok(86_400),
        "week" => Ok(604_800),
        "month" => Ok(MONTH_WINDOW_SECONDS),
        s => s
            .parse::<i32>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or(FormError::NoLength { index: index + 1 }),
    }
}

fn parse_window(
    fields: &[(String, String)],
    index: usize,
) -> Result<Option<QuotaWindow>, FormError> {
    let key = |f: &str| format!("w{index}_{f}");
    let Some(subject) = first(fields, &key("subject")) else {
        return Ok(None);
    };
    if !SUBJECTS.contains(&subject) {
        return Err(FormError::UnknownSubject {
            index: index + 1,
            subject: subject.to_owned(),
        });
    }
    // Why: the period arrives twice — the select and, behind it, the custom
    // seconds field of the same name. "custom" is the select saying "read
    // the next one".
    let raw = all(fields, &key("window"))
        .into_iter()
        .find(|v| v != "custom")
        .ok_or(FormError::NoLength { index: index + 1 })?;
    let window = QuotaWindow {
        window_seconds: window_seconds(&raw, index)?,
        subject: subject.to_owned(),
        max_requests: optional_i64(fields, &key("max_requests"))?,
        max_input_tokens: optional_i64(fields, &key("max_input_tokens"))?,
        max_output_tokens: optional_i64(fields, &key("max_output_tokens"))?,
        max_cost_microdollars: optional_dollars(fields, &key("max_cost"))?,
    };
    if window.max_requests.is_none()
        && window.max_input_tokens.is_none()
        && window.max_output_tokens.is_none()
        && window.max_cost_microdollars.is_none()
    {
        return Err(FormError::NoCeiling { index: index + 1 });
    }
    Ok(Some(window))
}

fn parse_mode(raw: Option<&str>, field: &str) -> Result<bool, FormError> {
    match raw.unwrap_or("enforce") {
        "warn" => Ok(true),
        "enforce" => Ok(false),
        other => Err(FormError::BadMode {
            field: field.to_owned(),
            value: other.to_owned(),
        }),
    }
}

fn parse_history(raw: Option<&str>) -> Result<SafetyHistoryMode, FormError> {
    match raw.unwrap_or("off") {
        "off" => Ok(SafetyHistoryMode::Off),
        "audit" => Ok(SafetyHistoryMode::Audit),
        "block" => Ok(SafetyHistoryMode::Block),
        other => Err(FormError::BadHistory {
            value: other.to_owned(),
        }),
    }
}

fn parse_phrases(raw: Option<&str>) -> Option<Vec<String>> {
    let lines: Vec<String> = raw?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    (!lines.is_empty()).then_some(lines)
}

// Why: The form as a policy, or the first thing wrong with it in the operator's
// words.
pub fn parse_policy_form(fields: &[(String, String)]) -> Result<ParsedPolicy, FormError> {
    let name = first(fields, "name").ok_or(FormError::MissingName)?;
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(FormError::BadName);
    }
    let priority = first(fields, "priority").map_or(Ok(0), |p| {
        p.parse::<i32>().map_err(|source| FormError::NotWhole {
            field: "priority".to_owned(),
            source,
        })
    })?;

    let mut quota_windows = Vec::new();
    for index in 0..MAX_WINDOWS {
        if let Some(w) = parse_window(fields, index)? {
            quota_windows.push(w);
        }
    }

    let scanners = all(fields, "scanner");
    if let Some(bad) = scanners.iter().find(|s| !SCANNERS.contains(&s.as_str())) {
        return Err(FormError::UnknownScanner {
            scanner: bad.clone(),
        });
    }
    let heuristic = HeuristicConfig {
        phrases: parse_phrases(first(fields, "heuristic_phrases")),
        extra_phrases: Vec::new(),
        disable_builtin: false,
    };
    let spec = GatewayPolicySpec {
        quota_mode: if parse_mode(first(fields, "quota_mode"), "quota_mode")? {
            QuotaMode::Warn
        } else {
            QuotaMode::Enforce
        },
        quota_windows,
        safety: SafetyConfig {
            mode: if parse_mode(first(fields, "safety_mode"), "safety_mode")? {
                SafetyMode::Warn
            } else {
                SafetyMode::Enforce
            },
            scanners,
            heuristic,
            block_categories: all(fields, "block"),
            block_response_categories: all(fields, "block_response"),
            history: parse_history(first(fields, "history"))?,
        },
    };
    let parsed = ParsedPolicy {
        name: name.to_owned(),
        enabled: first(fields, "enabled") == Some("true"),
        priority,
        spec,
    };
    // Why: the same validation the sync apply runs, so a save never writes a
    // row the file form would have refused.
    GatewayPolicyConfig {
        policies: vec![GatewayPolicyEntry {
            name: parsed.name.clone(),
            enabled: parsed.enabled,
            priority: parsed.priority,
            spec: parsed.spec.clone(),
        }],
    }
    .validate()
    .map_err(FormError::Invalid)?;
    Ok(parsed)
}
