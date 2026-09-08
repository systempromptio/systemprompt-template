//! `/admin/governance/hooks` — the hooks declared in plugin YAMLs and a
//! snapshot of the events they fed into the governance pipeline.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::format::local_time;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{ConfiguredHook, MarketplaceContext, UserContext};

const RECENT_LIMIT: i64 = 50;

#[derive(Debug, Serialize)]
struct HooksKpiView {
    label: &'static str,
    value: String,
    note: &'static str,
    tone: &'static str,
}

#[derive(Debug, Serialize)]
struct GovernanceHooksContext {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    kpis: Vec<HooksKpiView>,
    configured: usize,
    configured_hooks: Vec<ConfiguredHookRow>,
    has_configured: bool,
    recent_events: Vec<RecentEventRow>,
    recent_count: usize,
    has_recent: bool,
    export_json: String,
}

#[derive(Debug, Serialize)]
struct ConfiguredHookRow {
    plugin_id: String,
    event: String,
    matcher: String,
    command: String,
    is_async: bool,
    async_label: &'static str,
    timeout_label: String,
}

#[derive(Debug, Serialize)]
struct RecentEventRow {
    kind: String,
    created_at: String,
    plugin_id: String,
    tool_name: String,
    user_id: UserId,
    user_url: String,
    status: String,
    status_tone: &'static str,
}

fn configured_row(hook: ConfiguredHook) -> ConfiguredHookRow {
    ConfiguredHookRow {
        plugin_id: hook.plugin_id.as_str().to_owned(),
        event: hook.event,
        matcher: hook.matcher,
        command: hook.command,
        is_async: hook.is_async,
        async_label: if hook.is_async { "async" } else { "sync" },
        timeout_label: hook
            .timeout_ms
            .map_or_else(|| "—".to_owned(), |ms| format!("{ms} ms")),
    }
}

// Why: the export is the artefact this page exists for — the settings block
// a developer pastes so their Claude Code session asks this instance before
// every tool call. It is generated against the configured external URL so it
// never names a placeholder host.
fn hooks_export_json() -> String {
    use systemprompt::models::Config;
    let base = Config::get().map_or_else(
        |_| String::new(),
        |c| c.api_external_url.trim_end_matches('/').to_owned(),
    );
    let export = serde_json::json!({
        "hooks": {
            "PreToolUse": [{
                "matcher": "*",
                "hooks": [{
                    "type": "http",
                    "url": format!("{base}/api/public/hooks/govern"),
                    "timeout": 10
                }]
            }],
            "PostToolUse": [{
                "matcher": "*",
                "hooks": [{
                    "type": "http",
                    "url": format!("{base}/api/public/hooks/track"),
                    "timeout": 10,
                    "async": true,
                    "event": "PostToolUse"
                }]
            }]
        }
    });
    serde_json::to_string_pretty(&export).unwrap_or_default()
}

fn status_tone(status: &str) -> &'static str {
    match status {
        "allow" => "ok",
        "deny" => "err",
        "" => "muted",
        _ => "info",
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(crate) async fn governance_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let services_path = super::get_services_path()?;

    let configured_hooks: Vec<ConfiguredHookRow> =
        repositories::marketplace::hooks::list_configured_hooks(&services_path, &user_ctx.roles)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "list_configured_hooks failed");
                Vec::new()
            })
            .into_iter()
            .map(configured_row)
            .collect();

    // Why: these degrade to zero on purpose: the configured-hooks list beside them
    // is read from YAML, not the database, so a failure here shows real hooks
    // above empty counts and the contradiction is visible. They are logged
    // because otherwise the failure would leave no trace in the page *or* the
    // log, and this page is where an operator comes to diagnose exactly that.
    let pretool_fired = repositories::governance::hook_events::count_pretool_fired_24h(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "count_pretool_fired_24h failed"))
        .unwrap_or(0);
    let posttool_fired = repositories::governance::hook_events::count_posttool_fired_24h(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "count_posttool_fired_24h failed"))
        .unwrap_or(0);
    let recent_events =
        repositories::governance::hook_events::recent_hook_events(&pool, RECENT_LIMIT)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "recent_hook_events failed"))
            .unwrap_or_default();

    let recent_events: Vec<RecentEventRow> = recent_events
        .into_iter()
        .map(|e| {
            let status = e.status.unwrap_or_default();
            RecentEventRow {
                kind: e.kind,
                created_at: local_time(e.created_at),
                plugin_id: e
                    .plugin_id
                    .map(|p| p.as_str().to_owned())
                    .unwrap_or_default(),
                tool_name: e.tool_name.unwrap_or_default(),
                user_url: format!("/admin/user?id={}", urlencoding::encode(e.user_id.as_str())),
                user_id: e.user_id,
                status_tone: status_tone(&status),
                status,
            }
        })
        .collect();

    let configured = configured_hooks.len();
    let ctx = GovernanceHooksContext {
        page: "governance-hooks",
        title: "Hooks",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Hooks"),
        ],
        kpis: vec![
            HooksKpiView {
                label: "Configured",
                value: configured.to_string(),
                note: "declared across plugin YAMLs",
                tone: "accent",
            },
            HooksKpiView {
                label: "PreToolUse · 24h",
                value: pretool_fired.to_string(),
                note: "asked the chain before a tool ran",
                tone: "accent",
            },
            HooksKpiView {
                label: "PostToolUse · 24h",
                value: posttool_fired.to_string(),
                note: "reported a tool result",
                tone: "ok",
            },
        ],
        configured,
        has_configured: configured > 0,
        configured_hooks,
        recent_count: recent_events.len(),
        has_recent: !recent_events.is_empty(),
        recent_events,
        export_json: hooks_export_json(),
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-hooks",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}
