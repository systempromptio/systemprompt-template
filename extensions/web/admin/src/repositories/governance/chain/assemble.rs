//! Top-level chain assembly: resolve the id, fan out the per-table fetches, and
//! roll the parts up into a single [`ChainEnvelope`].

use sqlx::PgPool;

use super::fetch::{
    fetch_decisions, fetch_events, fetch_requests, fetch_summary, fetch_transcript,
};
use super::resolve::resolve_session_id;
use super::{
    AiRequestSummary, ChainEnvelope, ChainIdentity, ChainTotals, ChainUsageEvent, DecisionStage,
};

fn compute_totals(
    decisions: &[DecisionStage],
    requests: &[AiRequestSummary],
    events: &[ChainUsageEvent],
) -> ChainTotals {
    let mut totals = ChainTotals {
        decision_count: decisions.len() as i64,
        deny_count: decisions.iter().filter(|d| d.decision == "deny").count() as i64,
        event_count: events.len() as i64,
        request_count: requests.len() as i64,
        ..ChainTotals::default()
    };
    for r in requests {
        totals.total_cost_microdollars += r.cost_microdollars;
        totals.total_input_tokens += i64::from(r.input_tokens.unwrap_or(0));
        totals.total_output_tokens += i64::from(r.output_tokens.unwrap_or(0));
    }
    totals
}

/// Fetch the full decision chain for an identifier.
///
/// `id` may be a `decision_id`, `request_id`, `trace_id`, or `session_id`. An
/// id that resolves to no session yields `Ok(None)`; only a query failure is
/// an `Err`.
pub async fn fetch_decision_chain(
    pool: &PgPool,
    id: &str,
) -> Result<Option<ChainEnvelope>, sqlx::Error> {
    let Some(session_id) = resolve_session_id(pool, id).await? else {
        return Ok(None);
    };

    let (decisions, slots) = fetch_decisions(pool, &session_id).await?;
    let (requests, trace_id) = fetch_requests(pool, &session_id).await?;
    let events = fetch_events(pool, &session_id).await?;
    let transcript = fetch_transcript(pool, &session_id).await?;
    let summary = fetch_summary(pool, &session_id).await?;

    let identity = ChainIdentity {
        user_id: slots.user_id,
        agent_id: slots.agent_id,
        agent_scope: slots.agent_scope,
    };

    let totals = compute_totals(&decisions, &requests, &events);

    Ok(Some(ChainEnvelope {
        trace_id,
        session_id,
        identity,
        decisions,
        requests,
        events,
        transcript,
        summary,
        totals,
    }))
}
