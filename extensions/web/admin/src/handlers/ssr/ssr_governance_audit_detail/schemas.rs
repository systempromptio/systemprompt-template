//! The request's two tool schemas and the session's stored artifacts.
//!
//! The schema view answers "what did the model actually see?": the client's
//! `tools` array beside the one the gateway sent upstream, paired by name,
//! with the Gemini declaration rules each client schema breaks. The rule hits
//! are recomputed on read — nothing persists the sanitizer's per-request
//! decisions — so they say what the sanitizer had to rewrite, not what it
//! did.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::analytics::requests::{RequestSchemaRow, find_request_schemas};
use crate::types::tool_schema_diff::{
    ToolSchemaPair, WireTool, gemini_rules_apply, pair_tools, wire_tools,
};

#[derive(Debug, Default, Serialize)]
pub(super) struct SchemaView {
    pub available: bool,
    pub provider: String,
    pub rules_apply: bool,
    pub client_count: usize,
    pub provider_count: usize,
    pub changed_count: usize,
    pub rule_hit_count: usize,
    pub has_changes: bool,
    pub request_sha256: Option<String>,
    pub prepared_sha256: Option<String>,
    pub request_bytes: Option<i32>,
    pub request_truncated: bool,
    pub client_json: String,
    pub provider_json: String,
    pub has_provider: bool,
    pub tools: Vec<ToolPairView>,
}

#[derive(Debug, Serialize)]
pub(super) struct ToolPairView {
    pub name: String,
    pub description: String,
    pub changed: bool,
    pub has_client: bool,
    pub has_provider: bool,
    pub client_json: String,
    pub provider_json: String,
    pub rule_hits: Vec<String>,
    pub rule_hit_count: usize,
    pub has_rule_hits: bool,
    pub tone: &'static str,
    pub label: &'static str,
}

// Why: a failed read degrades to an unavailable section rather than taking
// the audit page with it.
pub(super) async fn load_schemas(pool: &PgPool, ai_request_id: &str, provider: &str) -> SchemaView {
    match find_request_schemas(pool, ai_request_id).await {
        Ok(Some(row)) => schema_view(&row, provider),
        Ok(None) => SchemaView::default(),
        Err(e) => {
            tracing::warn!(error = %e, "find_request_schemas failed");
            SchemaView::default()
        },
    }
}

fn schema_view(row: &RequestSchemaRow, provider: &str) -> SchemaView {
    let client = row
        .offered_tools
        .as_ref()
        .map(wire_tools)
        .unwrap_or_default();
    let served = row
        .prepared_tools
        .as_ref()
        .map(wire_tools)
        .unwrap_or_default();
    let rules_apply = gemini_rules_apply(provider);
    let pairs = pair_tools(&client, &served, rules_apply);
    let tools: Vec<ToolPairView> = pairs.iter().map(pair_view).collect();
    let changed_count = tools.iter().filter(|t| t.changed).count();
    let rule_hit_count = tools.iter().map(|t| t.rule_hits.len()).sum();
    SchemaView {
        available: row.offered_tools.is_some() || row.prepared_tools.is_some(),
        provider: provider.to_owned(),
        rules_apply,
        client_count: client.len(),
        provider_count: served.len(),
        changed_count,
        rule_hit_count,
        has_changes: changed_count > 0 || rule_hit_count > 0,
        request_sha256: row.request_body_sha256.clone(),
        prepared_sha256: row.prepared_body_sha256.clone(),
        request_bytes: row.request_bytes,
        request_truncated: row.request_truncated,
        client_json: row.offered_tools.as_ref().map(pretty).unwrap_or_default(),
        provider_json: row.prepared_tools.as_ref().map(pretty).unwrap_or_default(),
        has_provider: row.prepared_tools.is_some(),
        tools,
    }
}

fn pair_view(pair: &ToolSchemaPair) -> ToolPairView {
    let (tone, label) = match (&pair.client, &pair.provider, pair.changed) {
        (Some(_), None, _) => ("warn", "not sent"),
        (None, Some(_), _) => ("warn", "provider only"),
        (_, _, true) => ("info", "rewritten"),
        (_, _, false) => ("muted", "as sent"),
    };
    ToolPairView {
        name: pair.name.clone(),
        description: pair
            .client
            .as_ref()
            .or(pair.provider.as_ref())
            .and_then(|t| t.description.clone())
            .unwrap_or_default(),
        changed: pair.changed,
        has_client: pair.client.is_some(),
        has_provider: pair.provider.is_some(),
        client_json: pair.client.as_ref().map(schema_json).unwrap_or_default(),
        provider_json: pair.provider.as_ref().map(schema_json).unwrap_or_default(),
        has_rule_hits: !pair.rule_hits.is_empty(),
        rule_hit_count: pair.rule_hits.len(),
        rule_hits: pair.rule_hits.clone(),
        tone,
        label,
    }
}

fn schema_json(tool: &WireTool) -> String {
    pretty(&tool.schema)
}

// JSON: any stored payload, rendered for a `<pre>`.
fn pretty(value: &serde_json::Value) -> String {
    // Why: a Value always serializes; if it ever did not, the page says so
    // in the block rather than showing an empty one.
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("<unserializable: {e}>"))
}
