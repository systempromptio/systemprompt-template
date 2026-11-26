//! `/admin/users/{id}` — one account, in six tabs.
//!
//! Every tab is a link
//! (`?tab=identity|membership|access|devices|sessions|usage`), so
//! a view is bookmarkable and the server renders only the tab the reader asked
//! for. Usage is the account's spend and activity, beside who they are.

use crate::env::AdminEnv;

mod access_connections;
mod access_overview;
mod access_view;
mod context;
mod fill;
mod load;
#[cfg(feature = "gateway")]
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

// Why: Conversations sits second, directly after Identity. What this person
// has actually been asking the models is the question an operator opens an
// account for.
const TABS: &[(&str, &str)] = &[
    ("identity", "Identity"),
    #[cfg(feature = "agents")]
    ("conversations", "Conversations"),
    ("membership", "Membership"),
    ("access", "Access & connections"),
    ("devices", "Devices"),
    ("sessions", "Sessions"),
    #[cfg(feature = "gateway")]
    ("usage", "Usage"),
];

#[derive(Debug, Deserialize)]
pub(crate) struct DetailQuery {
    pub tab: Option<String>,
    pub page: Option<i64>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn user_detail_page(
    Extension(env): Extension<AdminEnv>,
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

    let headline = load::load_headline(&pool, &user_id).await;
    let group_ids = headline
        .profile
        .as_ref()
        .map(|p| p.group_ids.clone())
        .unwrap_or_default();
    let project_ids = headline
        .profile
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
        header: header_view(
            &detail,
            &group_ids,
            &project_ids,
            headline.defaults.as_ref(),
        ),
        kpis: kpis_view(&headline, &detail, group_ids.len(), project_ids.len()),
        can_write: user_ctx.is_admin,
        is_self: user_ctx.user_id.as_str() == user_id.as_str(),
        export: view::export(&user_id, &user_ctx),
        identity: None,
        #[cfg(feature = "agents")]
        conversations: None,
        membership: None,
        access: None,
        devices: None,
        sessions: None,
        #[cfg(feature = "gateway")]
        usage: None,
    };

    let data = Box::pin(fill::fill_tab(
        &env,
        fill::TabRead {
            pool: &pool,
            user_id: &user_id,
            detail: &detail,
            tab,
            page,
            #[cfg(feature = "agents")]
            viewer: &user_ctx,
        },
        data,
    ))
    .await;

    Ok(super::super::render_typed_page(
        &engine,
        "user-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn kpis_view(
    #[cfg_attr(
        not(feature = "gateway"),
        expect(
            unused_variables,
            reason = "the headline's spend is the gateway member's"
        )
    )]
    headline: &load::Headline,
    detail: &crate::types::UserDetail,
    groups: usize,
    projects: usize,
) -> DetailKpiView {
    DetailKpiView {
        #[cfg(feature = "gateway")]
        spend: spend_kpis(&headline.summary),
        #[cfg(not(feature = "gateway"))]
        spend: crate::handlers::ssr::types::Absent,
        sessions: i64::try_from(detail.sessions.len()).unwrap_or(0),
        devices: 0,
        groups,
        projects,
    }
}

#[cfg(feature = "gateway")]
fn spend_kpis(summary: &repositories::users::usage::UserGatewayUsage) -> context::DetailSpendView {
    context::DetailSpendView {
        requests: crate::handlers::ssr::format::format_token_total(summary.requests),
        cost: crate::handlers::ssr::format::format_cost(summary.cost_microdollars),
        tokens: crate::handlers::ssr::format::format_token_total(summary.tokens),
    }
}

fn resolve_tab(raw: Option<&str>) -> &'static str {
    TABS.iter()
        .copied()
        .find(|(slug, _)| Some(*slug) == raw)
        .map_or("identity", |(slug, _)| slug)
}

fn tab_links(user_id: &UserId, active: &'static str) -> Vec<DetailTabView> {
    let encoded = urlencoding::encode(user_id.as_str());
    TABS.iter()
        .copied()
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
            .and_then(|d| d.primary_group_id.as_ref().map(ToString::to_string))
            .unwrap_or_else(|| group_ids.first().cloned().unwrap_or_else(|| "—".to_owned())),
        primary_project: defaults
            .and_then(|d| d.primary_project_id.as_ref().map(ToString::to_string))
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
