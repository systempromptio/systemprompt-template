//! `/groups/{id}/usage` — what this group's members spent and ran.
//!
//! One response rather than five endpoints: the group page renders the whole
//! thing at once, and splitting it would make five round trips to draw one
//! screen. The window is a preset day count because that is what the screen
//! offers.
//!
//! The queries are the shared people-container rollups, so this endpoint and
//! the server-rendered group page cannot disagree about a number.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::people_usage::breakdown::{
    ModelUsageRow, SkillUsageRow, ToolUsageRow, list_scope_top_models, list_scope_top_skills,
    list_scope_top_tools,
};
use crate::repositories::people_usage::{
    DailyRequests, LEADERBOARD_LIMIT, get_scope_usage, list_daily_requests,
};
use crate::repositories::scope::{Attribution, ScopeKind, ScopeQuery};
use crate::types::groups::GroupUsageSummary;

#[derive(Debug, Deserialize)]
pub(crate) struct UsageQuery {
    pub range: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupUsageResponse {
    pub group_id: String,
    pub range: String,
    pub summary: GroupUsageSummary,
    pub top_models: Vec<ModelUsageRow>,
    pub top_skills: Vec<SkillUsageRow>,
    pub top_tools: Vec<ToolUsageRow>,
    pub daily: Vec<DailyRequests>,
}

// Why: an unrecognised range is 30 days rather than a 400. The value comes
// from a query string the dashboard writes, and a stale bookmark should draw
// the default view, not an error page.
pub(crate) fn window_days(range: Option<&str>) -> (String, i32) {
    match range {
        Some("7d") => ("7d".to_owned(), 7),
        Some("90d") => ("90d".to_owned(), 90),
        _ => ("30d".to_owned(), 30),
    }
}

pub(crate) async fn get_group_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Query(query): Query<UsageQuery>,
) -> AdminResult<Response> {
    let (range, days) = window_days(query.range.as_deref());
    let usage = get_scope_usage(
        &pool,
        &ScopeQuery::new(ScopeKind::Group, Attribution::Member, &group_id, days),
    )
    .await?;
    let top_models = list_scope_top_models(
        &pool,
        &ScopeQuery::new(ScopeKind::Group, Attribution::Member, &group_id, days),
        LEADERBOARD_LIMIT,
    )
    .await?;
    let top_skills = list_scope_top_skills(
        &pool,
        &ScopeQuery::new(ScopeKind::Group, Attribution::Member, &group_id, days),
        LEADERBOARD_LIMIT,
    )
    .await?;
    let top_tools = list_scope_top_tools(
        &pool,
        &ScopeQuery::new(ScopeKind::Group, Attribution::Member, &group_id, days),
        LEADERBOARD_LIMIT,
    )
    .await?;
    let daily = list_daily_requests(
        &pool,
        &ScopeQuery::new(ScopeKind::Group, Attribution::Member, &group_id, days),
    )
    .await?;
    Ok(Json(GroupUsageResponse {
        group_id,
        range,
        summary: GroupUsageSummary {
            requests: usage.requests,
            tokens: usage.tokens,
            cost_microdollars: usage.cost_microdollars,
            active_users: usage.active_members,
        },
        top_models,
        top_skills,
        top_tools,
        daily,
    })
    .into_response())
}
