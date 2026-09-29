//! The request's two tool schemas and the session's stored artifacts.
//!
//! The schema view answers "what did the model actually see?": the client's
//! `tools` array beside the one the gateway sent upstream, paired by name,
//! with the Gemini declaration rules each client schema breaks. The rule hits
//! are recomputed on read — nothing persists the sanitizer's per-request
//! decisions — so they say what the sanitizer had to rewrite, not what it
//! did. The artifacts list is every stored tool result of the same session,
//! which is how an oversize CLI result that overflowed into its own artifact
//! is found from the request that asked for it.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::repositories::analysis::tools::{
    ToolActivityFilter, ToolActivityPage, ToolActivityRow, ToolBreakdownBy, ToolSort,
    load_tool_activity_page,
};
use crate::repositories::analytics::requests::{RequestSchemaRow, find_request_schemas};
use crate::types::tool_schema_diff::{
    ToolSchemaPair, WireTool, gemini_rules_apply, pair_tools, wire_tools,
};

const ARTIFACT_LIMIT: i64 = 50;

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

#[derive(Debug, Default, Serialize)]
pub(super) struct ArtifactsView {
    pub rows: Vec<ArtifactLinkView>,
    pub has_rows: bool,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ArtifactLinkView {
    pub artifact_key: String,
    pub url: String,
    pub tool_name: String,
    pub server_name: String,
    pub artifact_type: String,
    pub title: String,
    pub bytes: String,
    pub request_id: Option<String>,
    pub is_this_request: bool,
    pub is_error: bool,
    pub is_structured: bool,
    pub created_at: String,
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

pub(super) async fn load_artifacts(
    pool: &PgPool,
    session_id: &SessionId,
    primary_request_id: Option<&str>,
) -> ArtifactsView {
    let filter = ToolActivityFilter {
        session: Some(session_id.as_str().to_owned()),
        artifacts_only: true,
        ..ToolActivityFilter::default()
    };
    let page = ToolActivityPage {
        sort: ToolSort::default(),
        descending: true,
        limit: ARTIFACT_LIMIT,
        offset: 0,
        breakdown: ToolBreakdownBy::default(),
    };
    let rows = match load_tool_activity_page(pool, &filter, page).await {
        Ok(result) => result.rows,
        Err(e) => {
            tracing::warn!(error = %e, "artifact list for the request page failed");
            Vec::new()
        },
    };
    let rows: Vec<ArtifactLinkView> = rows
        .iter()
        .filter_map(|r| artifact_link(r, primary_request_id))
        .collect();
    ArtifactsView {
        count: rows.len(),
        has_rows: !rows.is_empty(),
        rows,
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

fn artifact_link(
    r: &ToolActivityRow,
    primary_request_id: Option<&str>,
) -> Option<ArtifactLinkView> {
    let artifact_key = r.artifact_key.clone()?;
    Some(ArtifactLinkView {
        url: format!("/admin/artifacts/{}", urlencoding::encode(&artifact_key)),
        artifact_key,
        tool_name: r.tool_name.clone().unwrap_or_else(|| "—".to_owned()),
        server_name: r.server_name.clone().unwrap_or_default(),
        artifact_type: r.artifact_type.clone().unwrap_or_default(),
        title: r.artifact_title.clone().unwrap_or_default(),
        bytes: r
            .payload_bytes
            .map_or_else(|| "—".to_owned(), |b| b.to_string()),
        is_this_request: primary_request_id.is_some_and(|id| r.request_id.as_deref() == Some(id)),
        request_id: r.request_id.clone(),
        is_error: r.is_error,
        is_structured: r.is_structured,
        created_at: r
            .occurred_at
            .map(|t| {
                t.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default(),
    })
}
