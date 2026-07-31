//! SSR demo trace: what the governed coding agent actually did, in order.
//!
//! The decisions ledger answers "what has policy denied lately" across the
//! whole deployment. This page answers a narrower question that a demo needs:
//! for THIS agent session, show the prompt gate, the tool gate, the route
//! gate, the provider calls, and the tool fires as one timeline, so a denial
//! and the model call it prevented sit next to each other rather than in two
//! different tables.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, SessionId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::governance::demo_trace;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use view::{SessionView, TurnView, to_session_views, to_turn_views};

const PI_AGENT_ID: &str = "pi_agent";
const SESSION_LIMIT: i64 = 25;
const TRACE_LIMIT: i64 = 300;

#[derive(Debug, Deserialize)]
pub(crate) struct DemoTraceQuery {
    session: Option<String>,
}

#[derive(Debug, Serialize)]
struct DemoTraceContext {
    page: &'static str,
    title: &'static str,
    hero_title: &'static str,
    hero_subtitle: &'static str,
    sessions: Vec<SessionView>,
    has_sessions: bool,
    session_id: SessionId,
    session_detail_url: String,
    turns: Vec<TurnView>,
    has_rows: bool,
    prompts_blocked: usize,
    tools_blocked: usize,
    model_calls: usize,
}

pub(crate) async fn demo_trace_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<DemoTraceQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let sessions = demo_trace::list_demo_sessions(&pool, &AgentId::new(PI_AGENT_ID), SESSION_LIMIT)
        .await
        .map_err(AdminError::from)?;

    let selected = params
        .session
        .filter(|s| !s.is_empty())
        .map(SessionId::new)
        .or_else(|| sessions.first().map(|s| s.session_id.clone()));

    let rows = match selected.as_ref() {
        Some(session) => demo_trace::list_demo_trace(&pool, session, TRACE_LIMIT)
            .await
            .map_err(AdminError::from)?,
        None => Vec::new(),
    };

    let prompts_blocked = rows
        .iter()
        .filter(|r| r.kind == "prompt" && r.outcome == "deny")
        .count();
    let tools_blocked = rows
        .iter()
        .filter(|r| (r.kind == "tool" || r.kind == "route") && r.outcome == "deny")
        .count();
    let model_calls = rows.iter().filter(|r| r.kind == "request").count();

    let session_views = to_session_views(sessions, selected.as_ref());
    let turns = to_turn_views(rows);
    let session_id = selected.unwrap_or_else(|| SessionId::new(String::new()));

    let ctx = DemoTraceContext {
        page: "demo-trace",
        title: "Demo Trace",
        hero_title: "Demo Trace",
        hero_subtitle: "Every gate one agent session passed through, in order. Click any row \
                        for its full chain of custody.",
        has_sessions: !session_views.is_empty(),
        sessions: session_views,
        session_detail_url: super::entity_urls::session_detail_url(&session_id),
        session_id,
        has_rows: !turns.is_empty(),
        turns,
        prompts_blocked,
        tools_blocked,
        model_calls,
    };

    Ok(super::render_typed_page(
        &engine,
        "demo-trace",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}
