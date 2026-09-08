//! `/admin/users/{id}` — one account, in six tabs.
//!
//! Every tab is a link
//! (`?tab=identity|membership|access|devices|sessions|usage`), so
//! a view is bookmarkable and the server renders only the tab the reader asked
//! for. Usage is the page that used to live at `/admin/analytics/users/{id}`,
//! folded in: it was the same account seen from a different sidebar entry, and
//! the two are now one URL.

mod context;
mod load;
mod usage;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{DetailKpiView, DetailTabView, UserDetailContext, UserHeaderView};

use super::BASE_URL;

const TABS: [(&str, &str); 6] = [
    ("identity", "Identity"),
    ("membership", "Membership"),
    ("access", "Access"),
    ("devices", "Devices"),
    ("sessions", "Sessions"),
    ("usage", "Usage"),
];

#[derive(Debug, Deserialize)]
pub(crate) struct DetailQuery {
    pub tab: Option<String>,
    pub page: Option<i64>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    Query(query): Query<DetailQuery>,
) -> AdminHtmlResult<Response> {
    // Why: a non-console caller may open their own page and nobody else's.
    // Refusing by identity rather than by role keeps the profile link working
    // for a plain user without opening the roster to them.
    if !user_ctx.is_console && user_ctx.user_id.as_str() != user_id_raw {
        return Err(AdminError::Forbidden("You can only view your own profile.".to_owned()).into());
    }

    let user_id = UserId::new(user_id_raw.as_str());
    // Why: `Err` and `Ok(None)` must not collapse: this value alone decides
    // whether the page 404s, so a failed query would tell an admin the account
    // had been deleted. Only a genuine absence is a not-found.
    let Some(detail) = repositories::users::queries::find_user_detail(&pool, &user_id).await?
    else {
        return Err(AdminError::NotFound("No such user.".to_owned()).into());
    };

    let tab = resolve_tab(query.tab.as_deref());
    let page = query.page.unwrap_or(0).max(0);

    let (profile, summary, defaults) = tokio::join!(
        repositories::users::queries::find_user_access_profile(&pool, &user_id),
        repositories::users::usage::get_ai_request_summary(&pool, &user_id),
        repositories::scope::defaults::find_scope_defaults(&pool, &user_id),
    );
    let profile = profile
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: access profile unavailable"))
        .ok()
        .flatten();
    let summary = summary.unwrap_or_default();
    let defaults = defaults.ok().flatten();

    let group_ids = profile
        .as_ref()
        .map(|p| p.group_ids.clone())
        .unwrap_or_default();
    let project_ids = profile
        .as_ref()
        .map(|p| p.project_ids.clone())
        .unwrap_or_default();

    let data = UserDetailContext {
        page: "user-detail",
        title: display_name(&detail),
        breadcrumbs: vec![
            BreadcrumbView::link("Users", BASE_URL),
            BreadcrumbView::current(display_name(&detail)),
        ],
        tabs: tab_links(&user_id, tab),
        tab,
        header: header_view(&detail, &group_ids, &project_ids, defaults.as_ref()),
        kpis: DetailKpiView {
            requests_display: crate::handlers::ssr::format::format_token_total(summary.requests),
            cost_display: crate::handlers::ssr::format::format_cost(summary.cost_microdollars),
            tokens_display: crate::handlers::ssr::format::format_token_total(summary.tokens),
            sessions: i64::try_from(detail.sessions.len()).unwrap_or(0),
            devices: 0,
            groups: group_ids.len(),
            projects: project_ids.len(),
        },
        can_write: user_ctx.is_admin,
        is_self: user_ctx.user_id.as_str() == user_id.as_str(),
        identity: None,
        membership: None,
        access: None,
        devices: None,
        sessions: None,
        usage: None,
    };

    let data = Box::pin(fill_tab(&pool, &user_id, &detail, tab, page, data)).await;

    Ok(super::super::render_typed_page(
        &engine,
        "user-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: only the active tab is loaded. Rendering all six would make opening a
// person's Identity tab pay for their whole request history.
#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
async fn fill_tab(
    pool: &PgPool,
    user_id: &UserId,
    detail: &crate::types::UserDetail,
    tab: &'static str,
    page: i64,
    mut data: UserDetailContext,
) -> UserDetailContext {
    match tab {
        "membership" => {
            data.membership = Some(view::membership_tab(
                load::load_membership(pool, user_id).await,
            ));
        },
        "access" => {
            data.access = Some(view::access_tab(
                load::load_access(pool, user_id).await,
                user_id,
            ));
        },
        "devices" => {
            let rows = load::load_devices(pool, user_id).await;
            let tab_view = view::devices_tab(rows);
            data.kpis.devices = i64::try_from(tab_view.count).unwrap_or(0);
            data.devices = Some(tab_view);
        },
        "sessions" => {
            let rows = load::load_sessions(pool, user_id).await;
            data.sessions = Some(view::sessions_tab(
                &rows,
                user_id,
                page,
                load::SESSION_PAGE_SIZE,
            ));
        },
        "usage" => {
            data.usage = Some(view::usage_tab(
                load::load_usage(pool, user_id).await,
                user_id,
            ));
        },
        _ => {
            let identity = load::load_identity(pool, user_id, &detail.roles).await;
            let choices = super::scope_data::role_choices(&identity.roles);
            data.identity = Some(view::identity_tab(detail, &identity, choices));
        },
    }
    data
}

fn resolve_tab(raw: Option<&str>) -> &'static str {
    TABS.into_iter()
        .find(|(slug, _)| Some(*slug) == raw)
        .map_or("identity", |(slug, _)| slug)
}

fn tab_links(user_id: &UserId, active: &'static str) -> Vec<DetailTabView> {
    let encoded = urlencoding::encode(user_id.as_str());
    TABS.into_iter()
        .map(|(slug, label)| DetailTabView {
            slug,
            label,
            href: format!("/admin/users/{encoded}?tab={slug}"),
            is_active: slug == active,
        })
        .collect()
}

fn display_name(detail: &crate::types::UserDetail) -> String {
    detail
        .display_name
        .clone()
        .unwrap_or_else(|| detail.user_id.as_str().to_owned())
}

fn header_view(
    detail: &crate::types::UserDetail,
    group_ids: &[String],
    project_ids: &[String],
    defaults: Option<&repositories::scope::defaults::ScopeDefaults>,
) -> UserHeaderView {
    let name = display_name(detail);
    UserHeaderView {
        initials: name
            .split(|c: char| c.is_whitespace() || c == '-' || c == '.' || c == '@')
            .filter(|p| !p.is_empty())
            .take(2)
            .filter_map(|p| p.chars().next())
            .flat_map(char::to_uppercase)
            .collect(),
        email: detail
            .email
            .as_ref()
            .map(|e| e.as_str().to_owned())
            .unwrap_or_default(),
        has_roles: !detail.roles.is_empty(),
        roles: detail.roles.clone(),
        status_label: if detail.is_active {
            "Active"
        } else {
            "Suspended"
        },
        status_tone: if detail.is_active { "ok" } else { "muted" },
        is_active: detail.is_active,
        created_at: view::stamp(Some(detail.created_at)),
        last_active: view::stamp(detail.last_active),
        primary_group: defaults
            .and_then(|d| d.primary_group_id.clone())
            .unwrap_or_else(|| group_ids.first().cloned().unwrap_or_else(|| "—".to_owned())),
        primary_project: defaults
            .and_then(|d| d.primary_project_id.clone())
            .unwrap_or_else(|| {
                project_ids
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "—".to_owned())
            }),
        name,
        user_id: detail.user_id.clone(),
    }
}
