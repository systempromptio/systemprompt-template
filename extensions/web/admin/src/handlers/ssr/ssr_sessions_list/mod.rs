//! `/admin/entities/sessions` — the sessions list.
//!
//! A session is the unit an operator actually recognises: one interactive run,
//! holding every context, trace and AI request it produced. This page lists
//! them for the whole install and hands each one to
//! `ssr_session_detail`; it does not describe the caller, which is what the
//! page used to do instead.
//!
//! Bound to the same `?preset=&from=&to=&user_id=&error_only=&page=` contract
//! as the other filtered list pages — see `handlers::ssr::list_view`.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::{format_cost, format_token_total, short_id};
use crate::handlers::ssr::list_view;
use crate::repositories::analytics::sessions_list::{
    SessionListFilter, SessionListKpis, SessionPage, get_session_list_kpis, list_sessions_paged,
};
use crate::repositories::governance::filter_options::get_filter_options;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod context;
mod rows;

use context::{
    CurrentSessionView, FilterRibbon, SessionFilterOptionsView, SessionsListPageContext, StatsView,
};

const BASE_URL: &str = "/admin/entities/sessions";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct SessionListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<UserId>,
    pub error_only: Option<String>,
    pub page: Option<i64>,
}

pub(crate) async fn sessions_list_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SessionListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let page = query.page.unwrap_or(0).max(0);

    let ctx = load_sessions_data(&pool, &query, &user_ctx, range, page).await;
    Ok(super::render_typed_page(
        &engine, "sessions", &ctx, &user_ctx, &mkt_ctx,
    ))
}

async fn load_sessions_data(
    pool: &PgPool,
    query: &SessionListQuery,
    user_ctx: &UserContext,
    range: TimeRange,
    page: i64,
) -> SessionsListPageContext {
    let error_only = query.error_only.as_deref() == Some("true");
    let filter = SessionListFilter {
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        error_only,
    };
    let session_page = SessionPage {
        limit: PAGE_SIZE,
        offset: page * PAGE_SIZE,
    };

    let (list_res, kpis_res, options_res) = tokio::join!(
        list_sessions_paged(pool, &filter, range, session_page),
        get_session_list_kpis(pool, &filter, range),
        get_filter_options(pool, range),
    );

    let (items, total) = list_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_sessions_paged failed");
        (Vec::new(), 0)
    });
    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_session_list_kpis failed");
        SessionListKpis::default()
    });
    let options = options_res.unwrap_or_default();

    let preset = list_view::preset_str(
        &TimeRangeQuery {
            from: query.from.clone(),
            to: query.to.clone(),
            preset: query.preset.clone(),
        },
        range,
    );
    let selected_user = query.user_id.as_ref().map(UserId::as_str);
    let error_flag = query.error_only.as_deref();
    let pairs: [(&str, Option<&str>); 6] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", selected_user),
        ("error_only", error_flag),
        ("page", None),
    ];

    let session_rows: Vec<_> = items.iter().map(rows::session_row).collect();
    let has_sessions = !session_rows.is_empty();
    let shown_rows = i64::try_from(session_rows.len()).unwrap_or(i64::MAX);

    SessionsListPageContext {
        page: "sessions",
        title: "Sessions",
        current: current_session_view(user_ctx),
        time_range: list_view::time_range_context(BASE_URL, range, &preset),
        filter_ribbon: FilterRibbon {
            base_url: BASE_URL,
            preserved: list_view::build_preserved(range, &preset, &[("error_only", error_flag)]),
            options: SessionFilterOptionsView {
                users: list_view::annotate_group(&options.users, selected_user),
            },
            chips: list_view::build_chips(BASE_URL, &pairs, &[("user_id", "User")]),
        },
        stats: stats_view(&kpis),
        sessions: session_rows,
        has_sessions,
        total_count: total,
        pagination: list_view::build_pagination(
            BASE_URL,
            &pairs,
            list_view::PageWindow::new(page, PAGE_SIZE, total, shown_rows, "sessions"),
        ),
        error_only,
        error_toggle_url: error_toggle_url(&pairs, error_only),
    }
}

fn stats_view(kpis: &SessionListKpis) -> StatsView {
    StatsView {
        total_sessions: kpis.total_sessions,
        error_sessions: kpis.error_sessions,
        total_requests: kpis.total_requests,
        total_tool_uses: kpis.total_tool_uses,
        tokens_display: format_token_total(kpis.total_tokens),
        cost_display: format_cost(kpis.total_cost_microdollars),
    }
}

fn error_toggle_url(pairs: &list_view::QueryPairs<'_>, active: bool) -> String {
    let qs = list_view::query_string(pairs, &["error_only", "page"]);
    let base = if qs.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{qs}")
    };
    if active {
        base
    } else if qs.is_empty() {
        format!("{BASE_URL}?error_only=true")
    } else {
        format!("{base}&error_only=true")
    }
}

fn current_session_view(user_ctx: &UserContext) -> CurrentSessionView {
    let session_url = user_ctx.session_id.as_ref().map(session_detail_url);
    let session_id = user_ctx.session_id.clone();
    CurrentSessionView {
        username: user_ctx.username.clone(),
        session_id_short: session_id.as_ref().map(|s| short_id(s.as_str())),
        session_url,
        session_id,
    }
}
