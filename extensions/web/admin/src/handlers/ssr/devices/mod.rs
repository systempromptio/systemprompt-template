//! `/admin/devices` — every credential the estate has handed out, and whether
//! anything is still using it.
//!
//! Four tabs over four tables: bridge sessions (a heartbeat), personal access
//! tokens (a secret), device certificates (a key), and enrolment links (a
//! promise not yet claimed). Each tab is listed as people — one row per
//! person, their machines or credentials folded beneath it — because a flat
//! listing repeats the same name once per laptop, restart and token, and the
//! fleet reads as six of one colleague. The KPI strip above the tabs is
//! fleet-wide and does not change with the tab, because the question that
//! brings anyone here — how much of this is still alive — is answered before
//! a tab is chosen.
//!
//! Every control is a link. Tab, filter, sort and page are all query
//! parameters, so a view is bookmarkable, the back button works, and the
//! server renders exactly one tab's rows rather than four.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::devices::pats::CredentialQuery;
use crate::repositories::devices::sessions::SessionQuery;
use crate::repositories::devices::{sessions, stats};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod columns;
mod context;
mod credentials;
mod data;
mod view;

use columns::{BRIDGE_COLUMNS, CERT_COLUMNS, LINK_COLUMNS, PAT_COLUMNS};
use context::DevicesPageContext;
use view::ColumnSpec;

pub(crate) const BASE_URL: &str = "/admin/devices";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct DevicesQuery {
    tab: Option<String>,
    state: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    stale: Option<String>,
    page: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Bridges,
    Pats,
    Certs,
    Links,
}

impl Tab {
    fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("pats") => Self::Pats,
            Some("certs") => Self::Certs,
            Some("links") => Self::Links,
            _ => Self::Bridges,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Bridges => "bridges",
            Self::Pats => "pats",
            Self::Certs => "certs",
            Self::Links => "links",
        }
    }

    const fn items_label(self) -> &'static str {
        match self {
            Self::Bridges => "Machines",
            Self::Pats => "Tokens",
            Self::Certs => "Certificates",
            Self::Links => "Codes",
        }
    }

    const fn empty_message(self) -> &'static str {
        match self {
            Self::Bridges => {
                "No bridge has called home under this filter. Every desktop \
                 client writes a session on its first heartbeat."
            },
            Self::Pats => {
                "No access token matches this filter. Tokens are minted from \
                 the bridge setup page and from the profile page."
            },
            Self::Certs => {
                "No device certificate matches this filter. A certificate is \
                 enrolled when a machine completes the device-link flow."
            },
            Self::Links => {
                "Nothing is waiting to be linked. A connect code appears here \
                 for the ten minutes between issue and enrolment."
            },
        }
    }

    // Why: the sort key is matched against this list before it reaches SQL.
    // The statement binds it as a parameter rather than interpolating it, so
    // an unknown key is harmless — but falling back to the tab's default is
    // still the honest answer, because a header the reader cannot see cannot
    // be the one they clicked.
    const fn columns(self) -> &'static [ColumnSpec] {
        match self {
            Self::Bridges => BRIDGE_COLUMNS,
            Self::Pats => PAT_COLUMNS,
            Self::Certs => CERT_COLUMNS,
            Self::Links => LINK_COLUMNS,
        }
    }

    const fn default_sort(self) -> &'static str {
        match self {
            Self::Bridges => "heartbeat",
            Self::Pats | Self::Links => "created",
            Self::Certs => "enrolled",
        }
    }
}

fn resolve_sort(tab: Tab, raw: Option<&str>) -> &'static str {
    tab.columns()
        .iter()
        .find(|c| Some(c.key) == raw)
        .map_or_else(|| tab.default_sort(), |c| c.key)
}

fn resolve_state(raw: Option<&str>) -> &'static str {
    match raw {
        Some("active") => "active",
        Some("revoked") => "revoked",
        _ => "all",
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(crate) async fn devices_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DevicesQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = Tab::parse(query.tab.as_deref());
    let sort = resolve_sort(tab, query.sort.as_deref());
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    let state = resolve_state(query.state.as_deref());
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;
    let can_manage = user_ctx.is_admin;

    let fleet = stats::get_fleet_stats(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "device fleet stats failed"))
        .unwrap_or_default();
    let versions = sessions::list_version_counts(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge version histogram failed"))
        .unwrap_or_default();

    let credentials = CredentialQuery {
        state,
        sort,
        dir,
        limit: PAGE_SIZE,
        offset,
    };
    let (groups, users_total) = match tab {
        Tab::Bridges => {
            data::load_sessions(
                &pool,
                SessionQuery {
                    stale_only: query.stale.as_deref() == Some("1"),
                    sort,
                    dir,
                    limit: PAGE_SIZE,
                    offset,
                },
            )
            .await
        },
        Tab::Pats => credentials::load_pats(&pool, credentials, can_manage).await,
        Tab::Certs => credentials::load_certs(&pool, credentials, can_manage).await,
        Tab::Links => credentials::load_links(&pool, sort, dir, PAGE_SIZE, offset).await,
    };

    let shown = groups.len();
    let window = PageWindow::new(
        page,
        PAGE_SIZE,
        users_total,
        i64::try_from(shown).unwrap_or(PAGE_SIZE),
        "people",
    );
    let version_bars = view::build_version_bars(&versions);

    let ctx = DevicesPageContext {
        page: "devices",
        title: "Devices & enrolments",
        breadcrumbs: vec![
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Devices & enrolments"),
        ],
        tabs: view::build_tabs(&query, tab, &fleet),
        tab: tab.slug(),
        stats: view::build_stats(&query, &fleet),
        has_versions: !version_bars.is_empty(),
        versions: version_bars,
        filters: match tab {
            Tab::Bridges => view::build_stale_chips(&query, &fleet),
            // Why: a pending link has no state to filter on. It is waiting or
            // it has expired, and both are already listed.
            Tab::Links => Vec::new(),
            Tab::Pats => {
                view::build_state_chips(&query, state, (fleet.pats_total, fleet.pats_active))
            },
            Tab::Certs => {
                view::build_state_chips(&query, state, (fleet.certs_total, fleet.certs_active))
            },
        },
        sort_headers: view::build_sort_headers(&query, tab.columns(), sort, dir),
        items_label: tab.items_label(),
        count_label: format!("{users_total} people"),
        has_rows: shown > 0,
        empty_message: tab.empty_message(),
        groups,
        pagination: view::build_pagination(&query, window),
        can_manage,
        has_row_actions: can_manage && matches!(tab, Tab::Pats | Tab::Certs),
    };

    Ok(super::render_typed_page(
        &engine, "devices", &ctx, &user_ctx, &mkt_ctx,
    ))
}
