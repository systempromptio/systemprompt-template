//! `/admin/entities/contexts` — list of conversation contexts with per-user
//! drill-down. Filters by user, model, free text, and time range; supports a
//! flat "Contexts" view and a grouped "By user" summary view.

use std::sync::Arc;

use crate::repositories;
use crate::repositories::analytics_grp::contexts_list;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ContextsListQuery {
    pub user_id: Option<String>,
    pub model: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    pub view: Option<String>,
    pub limit: Option<i64>,
}

fn since_to_datetime(value: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    let dur = match value {
        "24h" | "1d" => Duration::hours(24),
        "7d" => Duration::days(7),
        "30d" => Duration::days(30),
        "90d" => Duration::days(90),
        _ => return None,
    };
    Some(now - dur)
}

fn microdollars_to_usd(micro: i64) -> f64 {
    (micro as f64) / 1_000_000.0
}

fn group_contexts_by_user(
    contexts: &[contexts_list::ContextListItem],
) -> std::collections::HashMap<String, Vec<serde_json::Value>> {
    let mut out: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for c in contexts {
        let key = c.user_id.clone().unwrap_or_else(|| "unknown".to_owned());
        out.entry(key).or_default().push(json!({
            "context_id":     c.context_id,
            "name":           c.name,
            "model":          c.model,
            "request_count":  c.request_count,
            "message_count":  c.message_count,
            "error_count":    c.error_count,
            "total_tokens":   c.total_input_tokens + c.total_output_tokens,
            "cost_usd":       microdollars_to_usd(c.total_cost_microdollars),
            "last_activity":  c.last_activity_at.map(|t| t.to_rfc3339()),
        }));
    }
    out
}

fn build_contexts_json(contexts: &[contexts_list::ContextListItem]) -> Vec<serde_json::Value> {
    contexts
        .iter()
        .map(|c| {
            json!({
                "context_id":     c.context_id,
                "name":           c.name,
                "user_id":        c.user_id,
                "display_name":   c.display_name,
                "session_id":     c.session_id,
                "model":          c.model,
                "request_count":  c.request_count,
                "message_count":  c.message_count,
                "error_count":    c.error_count,
                "input_tokens":   c.total_input_tokens,
                "output_tokens":  c.total_output_tokens,
                "total_tokens":   c.total_input_tokens + c.total_output_tokens,
                "cost_usd":       microdollars_to_usd(c.total_cost_microdollars),
                "first_request_at": c.first_request_at.map(|t| t.to_rfc3339()),
                "last_request_at":  c.last_request_at.map(|t| t.to_rfc3339()),
                "last_activity":    c.last_activity_at.map(|t| t.to_rfc3339()),
            })
        })
        .collect()
}

fn build_user_summaries_json(
    summaries: &[contexts_list::ContextUserSummary],
    by_user: &std::collections::HashMap<String, Vec<serde_json::Value>>,
) -> Vec<serde_json::Value> {
    summaries
        .iter()
        .map(|s| {
            let nested = by_user.get(&s.user_id).cloned().unwrap_or_default();
            json!({
                "user_id":        s.user_id,
                "display_name":   s.display_name,
                "context_count":  s.context_count,
                "request_count":  s.request_count,
                "message_count":  s.message_count,
                "input_tokens":   s.total_input_tokens,
                "output_tokens":  s.total_output_tokens,
                "total_tokens":   s.total_input_tokens + s.total_output_tokens,
                "cost_usd":       microdollars_to_usd(s.total_cost_microdollars),
                "error_count":    s.error_count,
                "last_activity":  s.last_activity_at.map(|t| t.to_rfc3339()),
                "models":         s.distinct_models,
                "contexts":       nested,
            })
        })
        .collect()
}

struct ContextsPageInputs {
    user_id: Option<String>,
    model: Option<String>,
    q: Option<String>,
    since_label: Option<String>,
    view: String,
    filter: contexts_list::ContextListFilter,
}

fn parse_inputs(params: ContextsListQuery) -> ContextsPageInputs {
    let trim_opt = |s: Option<String>| -> Option<String> {
        s.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
    };
    let user_id = trim_opt(params.user_id);
    let model = trim_opt(params.model);
    let q = trim_opt(params.q);
    let since_label = trim_opt(params.since);
    let since_dt = since_label.as_deref().and_then(since_to_datetime);
    let view = params
        .view
        .as_deref()
        .map(str::to_lowercase)
        .filter(|v| v == "users" || v == "contexts")
        .unwrap_or_else(|| "contexts".to_owned());
    let filter = contexts_list::ContextListFilter {
        user_id: user_id.clone(),
        model: model.clone(),
        free_text: q.clone(),
        since: since_dt,
        limit: params.limit.unwrap_or(0),
    };
    ContextsPageInputs {
        user_id,
        model,
        q,
        since_label,
        view,
        filter,
    }
}

struct ContextsPageData {
    contexts: Vec<contexts_list::ContextListItem>,
    user_summaries: Vec<contexts_list::ContextUserSummary>,
    kpis: contexts_list::ContextListKpis,
    models: Vec<String>,
    users_for_filter: Vec<crate::types::UserSummary>,
}

async fn fetch_page_data(
    pool: &PgPool,
    filter: &contexts_list::ContextListFilter,
) -> ContextsPageData {
    let contexts = contexts_list::fetch_context_list(pool, filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_context_list failed");
            Vec::new()
        });
    let user_summaries = contexts_list::fetch_context_user_summary(pool, filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_context_user_summary failed");
            Vec::new()
        });
    let kpis = contexts_list::fetch_context_list_kpis(pool, filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_context_list_kpis failed");
            contexts_list::ContextListKpis {
                total_contexts: 0,
                active_users: 0,
                total_requests: 0,
                total_messages: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost_microdollars: 0,
            }
        });
    let models = contexts_list::fetch_distinct_models(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_distinct_models failed");
            Vec::new()
        });
    let users_for_filter = repositories::list_users(pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_users failed in contexts page");
        Vec::new()
    });
    ContextsPageData {
        contexts,
        user_summaries,
        kpis,
        models,
        users_for_filter,
    }
}

fn build_page_json(inputs: &ContextsPageInputs, data: &ContextsPageData) -> serde_json::Value {
    let contexts_by_user = group_contexts_by_user(&data.contexts);
    let contexts_json = build_contexts_json(&data.contexts);
    let user_summaries_json = build_user_summaries_json(&data.user_summaries, &contexts_by_user);
    let users_for_filter_json: Vec<serde_json::Value> = data
        .users_for_filter
        .iter()
        .map(|u| {
            json!({
                "user_id":      u.user_id.to_string(),
                "display_name": u.display_name,
            })
        })
        .collect();
    let kpis = data.kpis;
    let total_tokens = kpis.total_input_tokens + kpis.total_output_tokens;
    let total_cost_usd = microdollars_to_usd(kpis.total_cost_microdollars);
    json!({
        "page": "contexts",
        "title": "Conversation Contexts",
        "contexts": contexts_json,
        "user_summaries": user_summaries_json,
        "users_for_filter": users_for_filter_json,
        "models": data.models,
        "kpis": {
            "total_contexts":   kpis.total_contexts,
            "active_users":     kpis.active_users,
            "total_requests":   kpis.total_requests,
            "total_messages":   kpis.total_messages,
            "total_tokens":     total_tokens,
            "total_cost_usd":   total_cost_usd,
        },
        "filter": {
            "user_id": inputs.user_id,
            "model":   inputs.model,
            "q":       inputs.q,
            "since":   inputs.since_label,
            "view":    inputs.view,
        },
        "view_is_users":    inputs.view == "users",
        "view_is_contexts": inputs.view == "contexts",
        "page_stats": [
            {"value": kpis.total_contexts, "label": "Contexts"},
            {"value": kpis.active_users, "label": "Users"},
            {"value": kpis.total_requests, "label": "Requests"},
            {"value": kpis.total_messages, "label": "Messages"},
        ],
    })
}

pub(crate) async fn skills_contexts_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ContextsListQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }
    let inputs = parse_inputs(params);
    let data = fetch_page_data(&pool, &inputs.filter).await;
    let payload = build_page_json(&inputs, &data);
    super::render_page(&engine, "skills-contexts", &payload, &user_ctx, &mkt_ctx)
}
