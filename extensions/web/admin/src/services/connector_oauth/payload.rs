//! Decode MCP tool envelopes without mistaking a site ID for a user identity.

use crate::error::{AdminError, AdminResult};
// JSON: remote MCP tools may return structured content or JSON text with
// wrapper objects.
use serde_json::Value;

// JSON: traverse only protocol/content wrappers, never arbitrary nested
// business records.
fn candidates(value: &Value, depth: usize, found: &mut Vec<Value>) {
    if depth > 6
        || value.get("isError").and_then(Value::as_bool) == Some(true)
        || value.get("error").is_some()
    {
        return;
    }
    found.push(value.clone());
    for key in [
        "structuredContent",
        "result",
        "data",
        "resources",
        "user",
        "account",
        "userInfo",
    ] {
        if let Some(inner) = value.get(key) {
            candidates(inner, depth + 1, found);
        }
    }
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        for item in content {
            if let Some(text) = item.get("text").and_then(Value::as_str)
                && let Ok(decoded) = serde_json::from_str(text)
            {
                candidates(&decoded, depth + 1, found);
            }
        }
    }
}

// JSON: only explicit Atlassian account identifiers establish user identity.
pub fn atlassian_user(value: &Value) -> AdminResult<Value> {
    let mut values = Vec::new();
    candidates(value, 0, &mut values);
    let mut selected: Option<(String, Value)> = None;
    for value in values {
        let id = value
            .get("account_id")
            .or_else(|| value.get("accountId"))
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty());
        if let Some(id) = id {
            if selected
                .as_ref()
                .is_some_and(|(previous, _)| previous != id)
            {
                return Err(AdminError::Unavailable(
                    "Atlassian returned conflicting account identities; connection was not saved"
                        .into(),
                ));
            }
            selected = Some((id.to_owned(), value));
        }
    }
    selected.map(|(_, value)| value).ok_or_else(|| {
        tracing::warn!(
            has_structured_content = value.get("structuredContent").is_some(),
            has_text_content = value.get("content").is_some(),
            "Atlassian identity response contains no recognized account identifier"
        );
        AdminError::Unavailable(
            "Atlassian identity verification returned no account ID; connection was not saved"
                .into(),
        )
    })
}

// JSON: accessible sites can be an array wrapped by
// structuredContent/result/data.
pub fn atlassian_sites(value: &Value) -> AdminResult<Vec<Value>> {
    let mut values = Vec::new();
    candidates(value, 0, &mut values);
    values
        .into_iter()
        .find_map(|v| {
            v.as_array()
                .filter(|sites| {
                    sites.iter().all(|site| {
                        site.get("id")
                            .or_else(|| site.get("cloudId"))
                            .and_then(Value::as_str)
                            .is_some_and(|id| !id.is_empty())
                    })
                })
                .map(|sites| {
                    sites
                        .iter()
                        .cloned()
                        .map(|mut site| {
                            if site.get("id").is_none() {
                                site["id"] = site["cloudId"].clone();
                            }
                            site
                        })
                        .collect()
                })
        })
        .ok_or_else(|| {
            AdminError::Unavailable(
                "Atlassian returned no recognizable accessible-site list".into(),
            )
        })
}
