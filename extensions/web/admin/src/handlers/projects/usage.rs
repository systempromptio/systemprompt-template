//! `/projects/{id}/usage` — what a project's members spent and ran.
//!
//! The group response minus the tool leaderboard: tools are entitled through
//! groups, so a project's tool mix answers a question nobody asks of a
//! project. The window presets are shared with the group handler so one
//! bookmark shape works on both screens, and so are the queries themselves.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::people_usage::breakdown::{
    ModelUsageRow, SkillUsageRow, list_scope_top_models, list_scope_top_skills,
};
use crate::repositories::people_usage::{
    DailyRequests, LEADERBOARD_LIMIT, get_scope_usage, list_daily_requests,
};
use crate::repositories::scope::{Attribution, ScopeKind, ScopeQuery};
use crate::types::groups::GroupUsageSummary;

use super::super::groups::usage::{UsageQuery, window_days};

#[derive(Debug, Serialize)]
pub(crate) struct ProjectUsageResponse {
    pub project_id: String,
    pub range: String,
    pub summary: GroupUsageSummary,
    pub top_models: Vec<ModelUsageRow>,
    pub top_skills: Vec<SkillUsageRow>,
    pub daily: Vec<DailyRequests>,
}

pub(crate) async fn get_project_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
    Query(query): Query<UsageQuery>,
) -> AdminResult<Response> {
    let (range, days) = window_days(query.range.as_deref());
    let usage = get_scope_usage(
        &pool,
        &ScopeQuery::new(ScopeKind::Project, Attribution::Member, &project_id, days),
    )
    .await?;
    let top_models = list_scope_top_models(
        &pool,
        &ScopeQuery::new(ScopeKind::Project, Attribution::Member, &project_id, days),
        LEADERBOARD_LIMIT,
    )
    .await?;
    let top_skills = list_scope_top_skills(
        &pool,
        &ScopeQuery::new(ScopeKind::Project, Attribution::Member, &project_id, days),
        LEADERBOARD_LIMIT,
    )
    .await?;
    let daily = list_daily_requests(
        &pool,
        &ScopeQuery::new(ScopeKind::Project, Attribution::Member, &project_id, days),
    )
    .await?;
    Ok(Json(ProjectUsageResponse {
        project_id,
        range,
        summary: GroupUsageSummary {
            requests: usage.requests,
            tokens: usage.tokens,
            cost_microdollars: usage.cost_microdollars,
            active_users: usage.active_members,
        },
        top_models,
        top_skills,
        daily,
    })
    .into_response())
}
