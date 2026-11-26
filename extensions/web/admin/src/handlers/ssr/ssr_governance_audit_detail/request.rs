//! The request half of the audit page: the gateway member's request log for
//! the session, the primary request, its evidence, schemas and artifacts.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::AiRequestId;

use crate::repositories::governance::chain::{AiRequestSummary, ChainEnvelope};

#[cfg(feature = "agents")]
use super::artifacts;
use super::{evidence, is_failed_status, rows, schemas};

// Why: everything on the page the gateway member's request log supplies.
// Flattened, so the template reads the same keys it always has.
#[derive(Debug, Serialize)]
pub(super) struct RequestPanels {
    evidence: evidence::EvidenceView,
    schemas: schemas::SchemaView,
    #[cfg(feature = "agents")]
    artifacts: artifacts::ArtifactsView,
    primary: Option<PrimaryRequest>,
    requests: Vec<rows::RequestRowView>,
}

#[derive(Debug, Serialize)]
pub(super) struct RequestUsage {
    request_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: String,
    cost_microdollars: i64,
    latency_ms: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct PrimaryRequest {
    id: String,
    request_id: AiRequestId,
    model: String,
    provider: String,
    status: String,
    is_failed: bool,
    error_message: Option<String>,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
    latency_ms: Option<i32>,
    cost_microdollars: i64,
    created_at_local: String,
}

pub(super) async fn load_request_panels(
    pool: &PgPool,
    envelope: &ChainEnvelope,
    primary: Option<&AiRequestSummary>,
) -> RequestPanels {
    let request_ids: Vec<String> = envelope.requests.iter().map(|r| r.id.clone()).collect();
    let primary_id = primary.map(|r| r.id.as_str());
    let primary_provider = primary
        .and_then(|r| r.provider.as_deref())
        .unwrap_or_default();
    let (evidence, schemas) = tokio::join!(
        evidence::load(pool, &request_ids),
        schemas::load_schemas(pool, primary_id.unwrap_or_default(), primary_provider),
    );
    RequestPanels {
        evidence,
        schemas,
        #[cfg(feature = "agents")]
        artifacts: artifacts::load_artifacts(pool, &envelope.session_id, primary_id).await,
        primary: primary.map(build_primary_json),
        requests: envelope
            .requests
            .iter()
            .map(|r| rows::build_request_row(r, primary.is_some_and(|p| p.id == r.id)))
            .collect(),
    }
}

pub(super) fn pick_primary<'a>(env: &'a ChainEnvelope, id: &str) -> Option<&'a AiRequestSummary> {
    env.requests
        .iter()
        .find(|r| r.id == id || r.request_id == id)
        .or_else(|| env.requests.first())
}

pub(super) fn build_usage(env: &ChainEnvelope) -> RequestUsage {
    let latency_ms = env
        .requests
        .iter()
        .filter_map(|r| r.latency_ms)
        .map(i64::from)
        .sum::<i64>();
    let cost_usd = env.totals.requests.total_cost_microdollars as f64 / 1_000_000.0;
    RequestUsage {
        request_count: env.totals.requests.request_count,
        input_tokens: env.totals.requests.total_input_tokens,
        output_tokens: env.totals.requests.total_output_tokens,
        cost_usd: format!("{cost_usd:.4}"),
        cost_microdollars: env.totals.requests.total_cost_microdollars,
        latency_ms,
    }
}

pub(super) fn build_primary_json(r: &AiRequestSummary) -> PrimaryRequest {
    PrimaryRequest {
        id: r.id.clone(),
        request_id: r.request_id.clone(),
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        provider: r.provider.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_failed: is_failed_status(&r.status),
        error_message: r.error_message.clone(),
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        latency_ms: r.latency_ms,
        cost_microdollars: r.cost_microdollars,
        created_at_local: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }
}
