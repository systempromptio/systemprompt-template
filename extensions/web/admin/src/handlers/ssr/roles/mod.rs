//! `/admin/roles` — who holds each role, and what holding it opens.
//!
//! The page is the role's side of two tables the rest of the console reads
//! from the user's side: `user_manual_roles` says who was granted what by
//! hand, and `access_control_rules` at the `role` band says what that grant
//! reaches. People are listed once each, with every role they hold. Reading is
//! the CONSOLE tier so a project manager can audit the estate; the grant and
//! revoke controls are MANAGE, and `platform_admin` narrows further to a
//! platform admin, which is the same rule `PUT /users/{id}/roles` enforces on
//! the write itself.

mod data;
mod rows;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::roles::{entitlements, members};
use crate::templates::AdminTemplateEngine;
use crate::types::role::{ROLES_PLATFORM, has_any};
use crate::types::{MarketplaceContext, Role, UserContext};

use data::RolesQuery;
use view::{HeaderFactView, RolesPageData};

pub(crate) async fn roles_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RolesQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let mut known: Vec<String> = Role::ALL.iter().map(|r| r.as_str().to_owned()).collect();
    known.extend(crate::repositories::users::queries::list_distinct_roles(&pool).await?);
    known.sort();
    known.dedup();
    let member_rows = members::list_role_holders(&pool, &known, data::MEMBER_CAP)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roles: member listing failed"))
        .unwrap_or_default();
    let entitlement_rows = entitlements::list_role_entitlements(&pool, data::ENTITLEMENT_CAP)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roles: entitlement listing failed"))
        .unwrap_or_default();

    let cards = rows::cards(&member_rows, &entitlement_rows, &query, &known);
    let matching = data::filtered_sorted(&member_rows, &query);
    let total = i64::try_from(matching.len()).unwrap_or(i64::MAX);
    let index = data::page_index(&query, total);
    let start = usize::try_from(index * data::PAGE_SIZE).unwrap_or(0);
    let page_rows: Vec<_> = matching
        .iter()
        .skip(start)
        .take(usize::try_from(data::PAGE_SIZE).unwrap_or(50))
        .cloned()
        .collect();
    let window = PageWindow::new(
        index,
        data::PAGE_SIZE,
        total,
        i64::try_from(page_rows.len()).unwrap_or(0),
        "people",
    );

    let page = RolesPageData {
        page: "roles",
        title: "Roles & permissions",
        can_write: user_ctx.is_admin,
        can_grant_platform_admin: has_any(&user_ctx.roles, ROLES_PLATFORM),
        facts: facts(&cards, &entitlement_rows, &member_rows),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Roles & permissions"),
        ],
        members: rows::member_views(&page_rows, &query),
        member_total: total,
        entitlement_total: entitlement_rows.len(),
        entitlements: rows::entitlement_views(&entitlement_rows),
        pagination: data::pagination(&query, window),
        sort_headers: rows::sort_headers(&query),
        role_options: rows::role_options(&query, &known),
        source_options: rows::source_options(&query),
        status_options: rows::status_options(&query),
        search: data::search(&query),
        known_roles: rows::role_options(&RolesQuery::default(), &known),
        filters_applied: query.any_applied(),
        clear_url: data::BASE_URL,
        cards,
    };

    Ok(super::render_typed_page(
        &engine, "roles", &page, &user_ctx, &mkt_ctx,
    ))
}

// Why: the five facts an operator checks before touching anyone's roles —
// how much admin there is, how much of it was granted by hand rather than by
// the directory, and whether the estate has the one platform admin it must
// never drop below. They sit on the header's meta line, not in a KPI band:
// the band worth the vertical space is the role tiles, which filter the table.
fn facts(
    cards: &[view::RoleCardView],
    entitlements: &[entitlements::RoleEntitlementRow],
    members: &[members::RoleHolderRow],
) -> Vec<HeaderFactView> {
    let manual: usize = members.iter().map(|m| m.manual_roles.len()).sum();
    let grants: usize = members.iter().map(|m| m.roles.len()).sum();
    let admins = cards
        .iter()
        .filter(|c| c.id == "admin" || c.id == "platform_admin")
        .map(|c| c.member_count)
        .sum::<usize>();
    let platform = cards
        .iter()
        .find(|c| c.id == "platform_admin")
        .map_or(0, |c| c.member_count);
    let denies = entitlements
        .iter()
        .filter(|e| matches!(e.access, crate::types::access_control::AccessDecision::Deny))
        .count();
    vec![
        HeaderFactView {
            value: members.len().to_string(),
            label: "people hold a role".to_owned(),
        },
        HeaderFactView {
            value: admins.to_string(),
            label: "on the admin plane".to_owned(),
        },
        HeaderFactView {
            value: platform.to_string(),
            label: if platform <= 1 {
                "platform admin, the last cannot be demoted".to_owned()
            } else {
                "platform admins".to_owned()
            },
        },
        HeaderFactView {
            value: manual.to_string(),
            label: format!("granted by hand, {} from the directory", grants - manual),
        },
        HeaderFactView {
            value: entitlements.len().to_string(),
            label: format!("role entitlements, {denies} of them denies"),
        },
    ]
}
