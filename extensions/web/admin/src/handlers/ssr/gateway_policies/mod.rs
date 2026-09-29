//! `/admin/gateway/policies` — the gateway policy editor: quota windows,
//! safety scanners, block lists and the warn/enforce switch of each plane,
//! written straight to `ai_gateway_policies`.
//!
//! Core reads that table on every inference request (a sixty-second
//! cache), so a save here is live without a restart. A plain form, no
//! JavaScript in the path: every field posts as `(name, value)` pairs and
//! [`parse_policy_form`] turns them into the spec the file would declare.
//! The Sync tab shows what this editor changed as drift on the
//! `gateway_policies` plane until it is exported or overwritten.

mod view;

use std::sync::Arc;

use axum::extract::{Form, Query, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;

use self::view::{GatewayPoliciesPageData, InForce, new_policy_view, policy_view};
use crate::activity::{self, NewActivity};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared::require_write_origin;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::sync_plane::{is_sync_tab, plane_card_by_id, sync_tab};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::gateway_policies::form::parse_policy_form;
use crate::repositories::gateway_policies::month_window_db::refresh_month_windows;
use crate::repositories::gateway_policies::rows::{
    PolicyWrite, delete_policy, list_policies, upsert_policy,
};

pub(crate) const BASE_URL: &str = "/admin/gateway/policies";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PoliciesQuery {
    pub saved: Option<String>,
    pub error: Option<String>,
    pub tab: Option<String>,
}

fn tabs(on_sync: bool) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "policies",
            label: "Policies",
            href: BASE_URL.to_owned(),
            is_active: !on_sync,
            count: None,
        },
        sync_tab(BASE_URL, on_sync),
    ]
}

fn require_console(page: &Page) -> Result<(), AdminError> {
    if page.user.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn require_admin_write(page: &Page, headers: &HeaderMap) -> Result<(), AdminError> {
    if !page.user.is_admin {
        return Err(AdminError::Forbidden(
            "Editing gateway policies needs an administrator.".to_owned(),
        ));
    }
    require_write_origin(headers)
}

// Why: which row core's merge lets win each section, so the page can say
// "this policy's windows are the live ones" rather than leave two enabled
// rows looking equally in force.
fn winners(
    rows: &[crate::repositories::gateway_policies::rows::PolicyRow],
) -> (Option<&str>, Option<&str>) {
    let mut windows = None;
    let mut safety = None;
    for r in rows.iter().filter(|r| r.enabled) {
        if !r.spec.quota_windows.is_empty() {
            windows = Some(r.name.as_str());
        }
        let s = &r.spec.safety;
        if !s.scanners.is_empty()
            || !s.block_categories.is_empty()
            || !s.block_response_categories.is_empty()
            || s.mode.is_warn()
        {
            safety = Some(r.name.as_str());
        }
    }
    (windows, safety)
}

pub(crate) async fn gateway_policies_page(
    page: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PoliciesQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&page)?;
    let on_sync = is_sync_tab(query.tab.as_deref());
    let sync = if on_sync {
        plane_card_by_id(
            &pool,
            crate::repositories::sync::gateway_policies::PLANE_ID,
            "",
        )
        .await?
    } else {
        None
    };
    let rows = list_policies(&pool).await.map_err(AdminError::Database)?;
    let today = chrono::Utc::now().date_naive();
    let (win_w, win_s) = winners(&rows);
    let policies = rows
        .iter()
        .map(|r| {
            policy_view(
                r,
                today,
                InForce {
                    windows: win_w == Some(r.name.as_str()),
                    safety: win_s == Some(r.name.as_str()),
                },
            )
        })
        .collect();
    let data = GatewayPoliciesPageData {
        page: "gateway-policies",
        title: "Gateway policies",
        tabs: tabs(on_sync),
        sync,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Gateway", "/admin/gateway"),
            BreadcrumbView::current("Policies"),
        ],
        can_write: page.user.is_admin,
        policies,
        new_policy: new_policy_view(),
        saved: query.saved,
        error: query.error,
        month_sentinel: GatewayPoliciesPageData::month_sentinel(),
        quotas_url: "/admin/governance/quotas",
        sync_url: "/admin/gateway/policies?tab=sync",
        docs_url: "/documentation/services-sync",
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &page.engine,
        "gateway-policies",
        &data,
        &page.user,
        &page.marketplace,
    ))
}

fn back_with(param: &str, value: &str) -> Redirect {
    Redirect::to(&format!(
        "{BASE_URL}?{param}={}",
        urlencoding::encode(value)
    ))
}

pub(crate) async fn save_gateway_policy(
    page: Page,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(fields): Form<Vec<(String, String)>>,
) -> AdminHtmlResult<Redirect> {
    require_admin_write(&page, &headers)?;
    let parsed = match parse_policy_form(&fields) {
        Ok(p) => p,
        Err(e) => return Ok(back_with("error", &e.to_string())),
    };
    upsert_policy(
        &pool,
        &PolicyWrite {
            name: &parsed.name,
            spec: &parsed.spec,
            enabled: parsed.enabled,
            priority: parsed.priority,
        },
    )
    .await?;
    // Why: a monthly window is saved with its sentinel; the rewrite makes it
    // live on today's value now rather than at the next daily run.
    refresh_month_windows(&pool, chrono::Utc::now()).await?;
    activity::record(
        &pool,
        NewActivity::gateway_policy_saved(&page.user.user_id, &parsed.name, false),
    )
    .await;
    Ok(back_with("saved", &parsed.name))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeleteForm {
    name: String,
}

pub(crate) async fn delete_gateway_policy(
    page: Page,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<DeleteForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin_write(&page, &headers)?;
    let name = form.name.trim();
    if name.is_empty() {
        return Ok(back_with("error", "a policy name is required to delete"));
    }
    let deleted = delete_policy(&pool, name)
        .await
        .map_err(AdminError::Database)?;
    if deleted == 0 {
        return Ok(back_with("error", &format!("no policy named '{name}'")));
    }
    activity::record(
        &pool,
        NewActivity::gateway_policy_saved(&page.user.user_id, name, true),
    )
    .await;
    Ok(back_with("saved", name))
}
