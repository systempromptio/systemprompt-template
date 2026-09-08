//! `/admin/mcp` and `/admin/mcp/{id}` — the MCP servers, what they are serving,
//! and who is on them.
//!
//! These are the only admin pages that read the MCP runtime tables, so they are
//! the only place an operator can see that a declared server has never
//! connected, or that traffic is arriving under a name nothing declares. Both
//! states are silent everywhere else.

mod columns;
mod detail;
mod rows;
mod sections;
mod view;


use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::mcp::runtime;
use crate::repositories::overview::liveness;
use crate::templates::AdminTemplateEngine;
use crate::types::{ENTITY_MCP_SERVER, MarketplaceContext, UserContext};

use super::super::ssr::ssr_helpers::render_typed_page;
use super::sorting::{direction, matches, preserved_search, sort_headers};
use super::view::assignment_counts_by_type;
use rows::{BASE_URL, RowInputs, Runtime, WINDOW_HOURS};
use view::{McpDetailData, McpPageData};


#[derive(Debug, Default, Deserialize)]
pub(crate) struct McpListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    // Why: filtering is a server round trip, not a class toggle — the count in
    // the toolbar, the empty state and the URL then cannot disagree with the
    // rows on screen.
    pub q: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct McpDetailQuery {
    pub page: Option<i64>,
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

// Why: all three reads are best-effort. A runtime table that will not answer
// must not take the page down with it — the declaration alone is still worth
// rendering, and the status column then says "never connected" rather than
// inventing a liveness it could not read.
async fn load_runtime(pool: &PgPool) -> Runtime {
    let heartbeat = liveness::list_mcp_server_liveness(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: heartbeat read failed"))
        .unwrap_or_default();
    let identities = runtime::list_mcp_proxy_identity_counts(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: identity read failed"))
        .unwrap_or_default();
    let activity = runtime::list_mcp_server_activity(pool, WINDOW_HOURS)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: activity read failed"))
        .unwrap_or_default();
    Runtime::new(heartbeat, identities, activity)
}

pub(crate) async fn mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<McpListQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = super::data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let rt = load_runtime(&pool).await;

    let mut ids: Vec<String> = catalog
        .mcp
        .iter()
        .map(|s| s.id.as_str().to_owned())
        .collect();
    for name in rt.names() {
        if !ids.contains(&name) {
            ids.push(name);
        }
    }

    let mut servers: Vec<view::McpServerRow> = ids
        .iter()
        .map(|id| {
            let server = catalog.mcp.iter().find(|s| s.id.as_str() == id);
            rows::build_row(&RowInputs {
                id,
                server,
                runtime: &rt,
                plugin_count: catalog.plugins_by_mcp.get(id).map_or(0, Vec::len),
                assignment_count: counts.get(id).copied().unwrap_or(0),
            })
        })
        .collect();

    let search = query.q.unwrap_or_default();
    let kpis = columns::kpis(&servers);
    servers.retain(|s| matches(&[&s.id, &s.description, &s.server_type], &search));
    let sort_key = query.sort.unwrap_or_else(|| "calls".to_owned());
    let sort_dir = direction(query.dir.as_deref());
    rows::sort_rows(&mut servers, &sort_key, sort_dir);

    let unconfigured_count = servers.iter().filter(|s| !s.configured).count();
    let page = McpPageData {
        page: "mcp",
        title: "MCP servers",
        subtitle: "Every tool server this instance declares, what it is serving right now, and who may reach it.",
        breadcrumbs: vec![BreadcrumbView::current("MCP servers")],
        window_label: "last 24 hours",
        heartbeat_label: format!(
            "alive = a session spoke within {} minutes",
            liveness::HEARTBEAT_INTERVAL_SECS * 2 / 60
        ),
        // Why: the tiles count the fleet, not the filtered view. A filter that
        // moved "alive now" would make the number mean two different things
        // depending on what was typed in the box above it.
        kpis,
        sort_headers: sort_headers(
            BASE_URL,
            &columns::columns(),
            &sort_key,
            sort_dir,
            &preserved_search(&search),
        ),
        servers_count: servers.len(),
        unconfigured_count,
        servers,
        access_control_url: "/admin/access-control?entity_type=mcp_server",
        sort_key,
        sort_dir: sort_dir.to_owned(),
        search,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-mcp",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn mcp_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(mcp_id): Path<String>,
    Query(query): Query<McpDetailQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = super::data::load_catalog(&path, &user_ctx.roles);
    let server = catalog.mcp.iter().find(|s| s.id.as_str() == mcp_id);
    let rt = load_runtime(&pool).await;
    let known_at_runtime = rt.heartbeat.contains_key(&mcp_id)
        || rt.identities.contains_key(&mcp_id)
        || rt.activity.contains_key(&mcp_id);

    // Why: a server the catalog does not declare but the runtime has served is
    // a real page, because the list links to it. Only a name neither half knows
    // is a 404.
    if server.is_none() && !known_at_runtime {
        return Err(AdminError::NotFound("No such MCP server.".to_owned()).into());
    }

    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let row = rows::build_row(&RowInputs {
        id: &mcp_id,
        server,
        runtime: &rt,
        plugin_count: catalog.plugins_by_mcp.get(&mcp_id).map_or(0, Vec::len),
        assignment_count: counts.get(&mcp_id).copied().unwrap_or(0),
    });

    let sections = sections::detail_sections(&pool, &mcp_id, query.page.unwrap_or(0).max(0)).await;

    let page = McpDetailData {
        page: "mcp",
        title: mcp_id.clone(),
        subtitle: row.description.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("MCP servers", BASE_URL),
            BreadcrumbView::current(mcp_id.clone()),
        ],
        configured: row.configured,
        enabled: row.enabled,
        status_label: row.status_label,
        status_tone: row.status_tone,
        window_label: "last 24 hours",
        kpis: columns::kpis(std::slice::from_ref(&row)),
        tools_count: sections.tools.len(),
        tools: sections.tools,
        executions_count: sections.executions_total,
        pagination: sections.pagination,
        executions: sections.executions,
        sessions_count: sections.sessions.len(),
        sessions: sections.sessions,
        grants_count: sections.grants.len(),
        grants: sections.grants,
        default_included: sections.default_included,
        config_facts: detail::config_facts(server),
        oauth_scopes: server.map(|s| s.oauth_scopes.clone()).unwrap_or_default(),
        included_by_count: catalog.plugins_by_mcp.get(&mcp_id).map_or(0, Vec::len),
        included_by: catalog
            .plugins_by_mcp
            .get(&mcp_id)
            .cloned()
            .unwrap_or_default(),
        matrix_url: row.matrix_url.clone(),
        access_control_url: "/admin/access-control",
        id: mcp_id,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-mcp-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
