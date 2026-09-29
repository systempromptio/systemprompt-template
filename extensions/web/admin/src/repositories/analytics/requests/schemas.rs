//! The tool schemas one request carried, as stored by the gateway.
//!
//! `offered_tools` is the client's `tools` array and `prepared_tools` is the
//! array that went upstream in the provider's wire shape; the two digests and
//! the request-body accounting say how much of the raw body survived the
//! audit payload cap. The page pairs the arrays itself
//! (`types::tool_schema_diff`).

// JSON: both arrays are protocol-boundary tool schemas in a provider's own
// wire shape; there is no one typed form they all share.
use serde_json::Value;
use sqlx::PgPool;

/// What `ai_request_payloads` keeps of one request's schemas.
#[derive(Debug, Clone)]
pub struct RequestSchemaRow {
    // JSON: the client's `tools` array, in the shape it sent.
    pub offered_tools: Option<Value>,
    // JSON: the `tools` array the gateway put upstream, provider-shaped.
    pub prepared_tools: Option<Value>,
    pub request_body_sha256: Option<String>,
    pub prepared_body_sha256: Option<String>,
    pub request_bytes: Option<i32>,
    pub request_truncated: bool,
}

pub async fn find_request_schemas(
    pool: &PgPool,
    ai_request_id: &str,
) -> Result<Option<RequestSchemaRow>, sqlx::Error> {
    sqlx::query_as!(
        RequestSchemaRow,
        r#"SELECT o.tools AS offered_tools, q.tools AS prepared_tools,
                  p.request_body_sha256, p.prepared_body_sha256,
                  p.request_bytes, p.request_truncated AS "request_truncated!"
           FROM ai_request_payloads p
           LEFT JOIN ai_tool_catalogs o ON o.sha256 = p.offered_tools_sha256
           LEFT JOIN ai_tool_catalogs q ON q.sha256 = p.prepared_tools_sha256
           WHERE p.ai_request_id = $1"#,
        ai_request_id
    )
    .fetch_optional(pool)
    .await
}
