//! `/admin/contexts` — every conversation on this instance, newest first.
//!
//! Two views of the same rows: "By user" (one row per person, their
//! conversations nested under it) and "All" (one conversation per row).
//! Filters by user, model, free text, time range and the shared
//! `?group=&project=` scope; `?side=1` includes conversations made of side
//! calls alone, which are hidden by default.

mod context;
mod load;
mod view;

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::repositories::analytics::conversation_rows::{
    ConversationFilter, ConversationSort, ConversationTotals,
};
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{ContextsPageContext, FilterView, ModelOptionView, PageKpisView, UserForFilterView};
use load::{ContextsPageData, load_page_data};

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ContextsListQuery {
    pub user_id: Option<UserId>,
    pub model: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    pub view: Option<String>,
    pub side: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    // Why: the shell's scope bar is the one window control on every AI-activity
    // page and it emits `?preset=`. This page's own parameter is `since`, so
    // the preset is read as a fallback rather than silently ignored.
    pub preset: Option<String>,
}

const BASE_URL: &str = "/admin/contexts";
const PAGE_SIZE: i64 = 50;

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

pub(super) struct ContextsPageInputs {
    user_id: Option<UserId>,
    model: Option<String>,
    q: Option<String>,
    since_label: Option<String>,
    view: String,
    pub(super) view_is_users: bool,
    pub(super) show_side: bool,
    pub(super) filter: ConversationFilter,
    pub(super) sort: ConversationSort,
    pub(super) descending: bool,
    pub(super) page: i64,
}

fn parse_inputs(params: &ContextsListQuery, scope: &SubjectScope) -> ContextsPageInputs {
    let trim_opt = |s: Option<String>| -> Option<String> {
        s.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
    };
    let user_id = params
        .user_id
        .clone()
        .filter(|u| !u.as_str().trim().is_empty());
    let model = trim_opt(params.model.clone());
    let q = trim_opt(params.q.clone());
    let raw_since = trim_opt(params.since.clone()).or_else(|| trim_opt(params.preset.clone()));
    let since_label = Some(match raw_since.as_deref() {
        Some("1d") => "24h".to_owned(),
        Some(v @ ("24h" | "7d" | "30d" | "90d" | "all")) => v.to_owned(),
        _ => "30d".to_owned(),
    });
    let since_dt = since_label.as_deref().and_then(since_to_datetime);
    let view = params
        .view
        .as_deref()
        .map(str::to_lowercase)
        .filter(|v| v == "users" || v == "all")
        .unwrap_or_else(|| "users".to_owned());
    let show_side = params.side.as_deref() == Some("1");
    let filter = ConversationFilter {
        user_id: user_id.clone(),
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
        model: model.clone(),
        free_text: q.clone(),
        since: since_dt,
        until: None,
        include_side_calls: show_side,
        error_only: false,
    };
    ContextsPageInputs {
        user_id,
        model,
        q,
        since_label,
        view_is_users: view == "users",
        view,
        show_side,
        filter,
        sort: ConversationSort::parse_conversation_sort(params.sort.as_deref()),
        descending: params.dir.as_deref() != Some("asc"),
        page: params.page.unwrap_or(0).max(0),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one call site; splitting the six inputs into a struct would only rename them"
)]
#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
async fn build_page_context(
    pool: &PgPool,
    user_ctx: &UserContext,
    request: &ScopeRequest,
    inputs: &ContextsPageInputs,
    data: &ContextsPageData,
    params: &ContextsListQuery,
) -> ContextsPageContext {
    let by_user = if inputs.view_is_users {
        view::group_by_user(&data.conversations)
    } else {
        HashMap::new()
    };
    let conversations: Vec<_> = if inputs.view_is_users {
        Vec::new()
    } else {
        data.conversations
            .iter()
            .map(view::conversation_item)
            .collect()
    };
    let user_summaries: Vec<_> = data
        .user_summaries
        .iter()
        .map(|s| view::user_summary(s, &by_user, params))
        .collect();
    let users_for_filter: Vec<UserForFilterView> = data
        .users_for_filter
        .iter()
        .map(|u| UserForFilterView {
            selected: inputs.user_id.as_ref() == Some(&u.user_id),
            user_id: u.user_id.clone(),
            display_name: u.display_name.clone(),
        })
        .collect();
    let models: Vec<ModelOptionView> = data
        .models
        .iter()
        .map(|m| ModelOptionView {
            selected: inputs.model.as_deref() == Some(m.as_str()),
            model: m.clone(),
        })
        .collect();
    let (count, shown, noun) = if inputs.view_is_users {
        (data.totals.users, data.user_summaries.len(), "users")
    } else {
        (
            data.total_conversations,
            data.conversations.len(),
            "conversations",
        )
    };
    let shown = i64::try_from(shown).unwrap_or(PAGE_SIZE);
    ContextsPageContext {
        page: "contexts",
        title: "Conversations",
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::current("Conversations"),
        ],
        has_conversations: !conversations.is_empty(),
        has_user_summaries: !user_summaries.is_empty(),
        conversations,
        user_summaries,
        users_for_filter,
        models,
        kpis: page_kpis(&data.totals),
        filter: FilterView {
            q: inputs.q.clone().unwrap_or_default(),
            since: inputs.since_label.clone().unwrap_or_default(),
            view: inputs.view.clone(),
            group: request.group.clone().unwrap_or_default(),
            project: request.project.clone().unwrap_or_default(),
            side: if inputs.show_side { "1" } else { "" }.to_owned(),
        },
        scope_filter: crate::handlers::ssr::list_view::scope_filter_view(
            pool,
            user_ctx,
            request,
            BASE_URL,
            vec![
                ("q".to_owned(), inputs.q.clone().unwrap_or_default()),
                ("model".to_owned(), inputs.model.clone().unwrap_or_default()),
                (
                    "user_id".to_owned(),
                    inputs
                        .user_id
                        .as_ref()
                        .map(|u| u.as_str().to_owned())
                        .unwrap_or_default(),
                ),
                (
                    "since".to_owned(),
                    inputs.since_label.clone().unwrap_or_default(),
                ),
                ("view".to_owned(), inputs.view.clone()),
                (
                    "side".to_owned(),
                    if inputs.show_side { "1" } else { "" }.to_owned(),
                ),
            ],
        )
        .await,
        view_tabs: view::view_tabs(params, &inputs.view),
        view_is_users: inputs.view_is_users,
        view_is_all: !inputs.view_is_users,
        pagination: view::build_pagination(
            params,
            PageWindow::new(inputs.page, PAGE_SIZE, count, shown, noun),
        ),
        sort_headers: view::build_sort_headers(params, inputs.sort, inputs.descending),
        total_count: data.total_conversations,
        count_label: format!("{count} {noun}"),
        show_side: inputs.show_side,
        side_toggle_url: view::side_toggle_url(params, inputs.show_side),
    }
}

pub(crate) async fn skills_contexts_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ContextsListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let request = ScopeRequest::from_query(
        &user_ctx,
        params.group.as_deref(),
        params.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let inputs = parse_inputs(&params, &scope);
    let data = load_page_data(&pool, &inputs, &scope).await?;
    let payload = build_page_context(&pool, &user_ctx, &request, &inputs, &data, &params).await;
    Ok(super::render_typed_page(
        &engine,
        "skills-contexts",
        &payload,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn page_kpis(t: &ConversationTotals) -> PageKpisView {
    PageKpisView {
        conversations: t.conversations,
        users: t.users,
        turns: t.turns,
        tool_calls: t.tool_calls,
        side_calls: t.side_calls,
        side_call_cost_display: format_cost(t.side_call_cost_microdollars),
        tokens_display: format_token_total(t.total_tokens),
        cost_display: format_cost(t.total_cost_microdollars),
    }
}
