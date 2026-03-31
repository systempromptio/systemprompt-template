use crate::admin::handlers::extract_user_from_cookie;
use crate::admin::numeric;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use crate::utils::html_escape;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use serde_json::json;

pub(crate) const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

pub(crate) mod charts;
mod ssr_add_passkey;
mod ssr_browse_plugins;
pub(crate) mod ssr_control_center;
mod ssr_dashboard;
mod ssr_demo_register;
mod ssr_dashboard_activity;
mod ssr_dashboard_helpers;
mod ssr_dashboard_report;
mod ssr_dashboard_traffic;
mod ssr_dashboard_traffic_pages;
mod ssr_dashboard_types;
mod ssr_access_control;
mod ssr_events;
mod ssr_governance;
mod ssr_gamification;
mod ssr_jobs;
mod ssr_marketplace;
mod ssr_my_activity;
mod ssr_my_agents;
mod ssr_my_hooks;
mod ssr_my_marketplace;
mod ssr_org_marketplace;
mod ssr_my_mcp_servers;
mod ssr_my_plugin_view;
mod ssr_my_plugins;
mod ssr_my_plugins_helpers;
mod ssr_my_secrets;
mod ssr_my_skills;
mod ssr_profile;
mod ssr_settings;
mod ssr_setup;
mod ssr_users;
mod ssr_agents;
mod ssr_hooks;
mod ssr_mcp;
mod ssr_plugins;
mod ssr_skills;
mod ssr_traces;
pub(crate) mod types;

pub(crate) use ssr_agents::{agents_page, agent_edit_page};
pub(crate) use ssr_hooks::{hooks_page, hook_edit_page};
pub(crate) use ssr_mcp::{mcp_servers_page, mcp_edit_page};
pub(crate) use ssr_plugins::plugins_page;
pub(crate) use ssr_skills::{skills_page, skill_edit_page};
pub(crate) use ssr_add_passkey::add_passkey_page;
pub(crate) use ssr_browse_plugins::browse_plugins_page;
pub(crate) use ssr_control_center::build_session_groups_with_status;
pub(crate) use ssr_control_center::control_center_page;
pub(crate) use ssr_control_center::handle_analyse_session;
pub(crate) use ssr_control_center::handle_batch_update_session_status;
pub(crate) use ssr_control_center::handle_generate_report;
pub(crate) use ssr_control_center::handle_rate_session;
pub(crate) use ssr_control_center::handle_rate_skill;
pub(crate) use ssr_control_center::handle_update_session_status;
pub(crate) use ssr_dashboard::dashboard_page;
pub(crate) use ssr_demo_register::demo_register_page;
pub(crate) use ssr_dashboard_report::handle_generate_traffic_report;
pub(crate) use ssr_events::events_page;
pub(crate) use ssr_gamification::achievements_page;
pub(crate) use ssr_gamification::leaderboard_page;
pub(crate) use ssr_jobs::jobs_page;
pub(crate) use ssr_marketplace::marketplace_versions_page;
pub(crate) use ssr_my_activity::my_activity_page;
pub(crate) use ssr_my_agents::{my_agent_edit_page, my_agents_page};
pub(crate) use ssr_my_hooks::my_hooks_page;
pub(crate) use ssr_my_marketplace::my_marketplace_page;
pub(crate) use ssr_org_marketplace::org_marketplace_page;
pub(crate) use ssr_my_mcp_servers::my_mcp_servers_page;
pub(crate) use ssr_my_plugin_view::my_plugin_view_page;
pub(crate) use ssr_my_plugins::{my_plugin_edit_page, my_plugins_page};
pub(crate) use ssr_my_secrets::my_secrets_page;
pub(crate) use ssr_my_skills::{my_skill_edit_page, my_skills_page};
pub(crate) use ssr_profile::handle_generate_profile_report;
pub(crate) use ssr_profile::profile_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_governance::governance_page;
pub(crate) use ssr_traces::traces_page;
pub(crate) use ssr_users::{user_detail_page, users_page};

fn demo_help_text(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "control-center" => Some((
            "The <strong>Control Center</strong> is your real-time operations hub. It shows live Claude sessions, APM (Actions Per Minute) metrics, conversation history, skill effectiveness ratings, and session health analytics. In a production deployment, this page streams live updates via SSE as your team uses Claude, giving you instant visibility into AI usage patterns and performance. The Control Center is fully functional in this demo &mdash; connect via <strong>Claude Code</strong> to see live session data populate in real time. Claude Code is the recommended integration for evaluation while Cowork (research preview) stabilises.",
            "dashboard",
        )),
        "profile" => Some((
            "<strong>Profile &amp; Insights</strong> analyses your usage patterns to identify your user archetype, strengths, and areas for improvement. It aggregates 30 days of Claude session data into behavioural analytics, comparing your usage against global averages. This helps teams understand how individuals interact with AI and where additional training or skills could help. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "profile",
        )),
        "dashboard" => Some((
            "The <strong>Admin Dashboard</strong> provides system-wide analytics for platform administrators. It shows web traffic metrics, MCP tool call success rates, user activity timelines, and content performance data. This is the admin-only overview of the entire platform's health and usage. In a production deployment, this dashboard tracks every Claude interaction across your organisation. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "dashboard",
        )),
        "my-plugins" | "plugins" => Some((
            "<strong>Plugins</strong> shows all plugins installed in your workspace. Plugins are the core building block &mdash; each bundles skills (prompt templates), agents (specialised Claude configurations), and MCP server connections into a single distributable package. You can customise any plugin's components, fork official plugins, or create your own from scratch. Enterprise deployments control which plugins are available per role and department. Plugins work best with <strong>Claude Code</strong>, the recommended integration &mdash; install your marketplace link to load all governed plugins into any Claude Code session. Cowork (research preview) support for plugin distribution is still maturing.",
            "plugins",
        )),
        "browse-plugins" => Some((
            "The <strong>Plugin Directory</strong> is where you discover and install plugins from the marketplace. Each plugin is a curated bundle of skills, agents, and integrations that extends Claude's capabilities for specific workflows &mdash; from code review to documentation to CRM automation. Browse by category, check ratings and usage stats, and install with one click. Once installed, plugins appear in My Plugins where you can customise them. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "browse-plugins",
        )),
        "my-skills" | "skills" => Some((
            "<strong>Skills</strong> manages the prompt templates and instructions that guide Claude's behaviour. Skills are the most granular customisation unit &mdash; each defines how Claude should handle a specific type of task, from writing code reviews to summarising documents. Skills can be shared across plugins, versioned, and tracked for usage frequency and effectiveness. In production, skills ensure consistent AI output quality across your entire team. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "skills",
        )),
        "my-agents" | "agents" => Some((
            "<strong>Agents</strong> shows specialised Claude configurations with defined roles and capabilities. Each agent has a dedicated system prompt, assigned skills, and tool access that shapes how Claude responds in specific contexts &mdash; a code reviewer agent behaves differently from a documentation agent. Agents track effectiveness ratings, usage patterns, and session counts over time, giving you data on which configurations work best. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "agents",
        )),
        "my-mcp-servers" | "mcp-servers" => Some((
            "<strong>MCP Servers</strong> displays the Model Context Protocol server connections in your workspace. MCP servers extend Claude's capabilities by providing external tools and data sources &mdash; database queries, API integrations, file operations, and more. Each server exposes typed tool endpoints that Claude can call during conversations, with authentication handled via API keys in My Secrets. All MCP tool calls are logged for audit and governance. MCP servers work best with <strong>Claude Code</strong>, the recommended integration &mdash; Claude Code natively supports MCP server connections via plugins. Cowork (research preview) support for external MCP servers is still maturing.",
            "mcp-servers",
        )),
        "my-secrets" => Some((
            "<strong>My Secrets</strong> manages API keys, tokens, and environment variables used by your plugins and MCP servers. Secrets are encrypted at rest, scoped per plugin, and injected at runtime &mdash; ensuring each integration only accesses its own credentials. Configure authentication for external services like databases, APIs, and SaaS tools here. Secret access is logged and governed by the same access control policies as other platform features. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "secrets",
        )),
        "my-hooks" | "hooks" => Some((
            "<strong>Hooks</strong> manages event handlers that fire during Claude Code sessions. Hooks intercept lifecycle events like PreToolUse, PostToolUse, SessionStart, and PermissionRequest, enabling custom behaviour at each stage &mdash; from analytics tracking and compliance logging to automated approvals and content filtering. Each hook specifies which events it listens to, optional matchers for filtering, and the action to execute. Hooks work best with <strong>Claude Code</strong>, the recommended integration &mdash; hooks fire natively during Claude Code sessions. Cowork (research preview) support for hooks is still maturing.",
            "hooks",
        )),
        "my-marketplace" | "org-marketplace" => Some((
            "<strong>Marketplace</strong> shows your published plugin collection &mdash; the installable bundle that distributes your governed AI capabilities. Share your marketplace link with team members or the broader community. It works with all Claude surfaces: Claude Code (CLI), Claude Desktop (Cowork), and claude.ai. When others install your marketplace, they receive your entire curated set of plugins, skills, agents, and MCP server connections, all governed by the access control policies you define. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "marketplace",
        )),
        "my-activity" => Some((
            "<strong>My Activity</strong> tracks your platform engagement across 13 activity categories &mdash; from skill usage and plugin installations to agent interactions and governance events. The activity log provides a chronological record of every interaction, while the achievements system offers milestone-based goals that reward consistent usage and feature exploration. Activity data feeds into your profile insights and leaderboard ranking. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "activity-tracking",
        )),
        "achievements" => Some((
            "<strong>Achievements</strong> displays all available milestones and their unlock status. Achievements are earned by using platform features &mdash; installing plugins, creating skills, running agents, configuring MCP servers, and maintaining usage streaks. Each achievement shows its rarity (how many users have unlocked it), XP reward, and unlock criteria. Achievements encourage feature discovery and consistent platform adoption across teams. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "achievements",
        )),
        "leaderboard" => Some((
            "The <strong>Leaderboard</strong> ranks users by XP, sessions, streaks, and other engagement metrics. It provides a community view of platform adoption, helping identify power users, track team-wide engagement, and encourage healthy competition. Department and team filters let managers see adoption within their groups. Users can opt out of leaderboard visibility in Settings. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "gamification",
        )),
        "settings" => Some((
            "<strong>Settings</strong> manages your account profile, notification preferences, and tier information. It shows your current usage against plan limits &mdash; plugins, skills, agents, MCP servers, hooks, and secrets. Enterprise integrations can customise which settings are available and enforce organisation-wide defaults. Tier upgrades unlock additional capacity and premium features. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "my-workspace",
        )),
        "users" => Some((
            "<strong>Users</strong> is the admin user management console. It shows all registered users with their activity metrics, XP rankings, session counts, roles, departments, and last active timestamps. Admins can create users, assign roles and departments, manage permissions, and monitor adoption across the organisation. User data drives access control policies and governance decisions. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "users",
        )),
        "events" => Some((
            "<strong>Events</strong> displays the system-wide event log &mdash; every Claude tool call, hook execution, session lifecycle event, and platform action across all users. Events can be filtered by type, user, time range, and severity, providing a full audit trail for compliance, debugging, and incident investigation. In production, events integrate with external SIEM systems for enterprise security workflows. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "events",
        )),
        "jobs" => Some((
            "<strong>Jobs</strong> monitors the background job queue &mdash; scheduled tasks including template compilation, analytics aggregation, content pre-rendering, marketplace sync, and activity scoring. Each job shows its cron schedule, last run status, execution duration, and next scheduled run. Failed jobs display error details for debugging. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "jobs",
        )),
        "marketplace" | "marketplace-versions" => Some((
            "The <strong>Marketplace</strong> is the public plugin browser where anyone can discover, evaluate, and install community plugins. Plugins are ranked by usage, ratings, and quality scores. Enterprise teams use the marketplace to publish internal plugins for their organisation, with visibility controls to keep proprietary capabilities private. Version history tracks every change, enabling rollback and audit. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "marketplace",
        )),
        "getting-started" => Some((
            "<strong>Getting Started</strong> helps new users select their initial plugins and configure their workspace. Choose from curated plugin sets based on your role &mdash; developer, analyst, manager, or custom &mdash; then refine your selection. Each plugin set bundles the skills, agents, and integrations most relevant to your workflow. Start by installing the <strong>Claude Code</strong> plugin &mdash; it is the recommended integration for evaluation while Cowork (research preview) stabilises.",
            "getting-started",
        )),
        "traces" => Some((
            "<strong>Trace Detail</strong> shows a timeline of every event in a single session &mdash; tool calls, governance decisions, MCP server interactions, and prompt submissions. Use the <code>session_id</code> query parameter to view a specific session. Events are ordered chronologically with delta timing between each step, giving you a waterfall view of the full execution pipeline. Connect via <strong>Claude Code</strong> to generate traced sessions, then inspect them here.",
            "events",
        )),
        "governance" => Some((
            "<strong>Governance</strong> provides oversight and policy management for AI usage across your organisation. Review every tool call decision (allowed, denied, or modified), configure usage policies per role and department, and set guardrails for how Claude is used by your team. Governance tracks compliance rates, policy violations, and cost allocation in real time. In production, governance policies are enforced automatically at the tool call level. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "tool-governance",
        )),
        "access-control" => Some((
            "<strong>Access Control</strong> manages permissions, roles, and authorisation policies across your organisation. Define which roles and departments can access specific plugins, agents, and MCP servers. Access control operates at the tool call level &mdash; when Claude invokes a governed tool, the platform checks the user's role and department against the configured policies in real time. Audit every permission change with full history. Connect via <strong>Claude Code</strong> for the best evaluation experience &mdash; it is the recommended integration while Cowork (research preview) stabilises.",
            "access-control",
        )),
        "setup" => Some((
            "<strong>Setup</strong> guides you through connecting Claude to your workspace. We recommend starting with <strong>Claude Code</strong>, the most stable and feature-complete integration for evaluation &mdash; it supports the full governance pipeline including plugins, hooks, MCP servers, and real-time analytics. Cowork (Desktop) is available as a research preview but has restrictions and shifting APIs that may limit functionality. Configure your webhook endpoint, install the Claude Code plugin with a single command, and verify the connection to start tracking conversations and enabling governance features.",
            "integration-claude-code",
        )),
        _ => None,
    }
}

fn branding_context(engine: &AdminTemplateEngine) -> serde_json::Value {
    match engine.branding() {
        Some(b) => json!({"branding": b}),
        None => json!({}),
    }
}

pub(crate) async fn login_page(Extension(engine): Extension<AdminTemplateEngine>) -> Response {
    match engine.render("login", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Login page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub(crate) async fn verify_pending_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    match engine.render("verify-pending", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Verify-pending page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub(crate) async fn register_page(
    headers: HeaderMap,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if extract_user_from_cookie(&headers).is_ok() {
        return Redirect::to("/control-center").into_response();
    }
    match engine.render("register", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Register page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub(crate) fn render_page(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &serde_json::Value,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    let mut merged = data.clone();
    if let Some(obj) = merged.as_object_mut() {
        obj.insert(
            "current_user".to_string(),
            json!({
                "user_id": user_ctx.user_id,
                "username": user_ctx.username,
                "roles": user_ctx.roles,
                "is_admin": user_ctx.is_admin,
            }),
        );
        let git_url = format!(
            "{}/api/public/marketplace/{}.git",
            mkt_ctx.site_url, mkt_ctx.user_id
        );
        let cowork_git_url = format!(
            "{}/api/public/marketplace/{}/cowork.git",
            mkt_ctx.site_url, mkt_ctx.user_id
        );
        let install_cmd = format!("/plugin marketplace add {git_url}");
        let mcp_url = format!(
            "{}/api/v1/mcp/skill-manager/mcp",
            mkt_ctx.site_url
        );
        obj.insert(
            "marketplace".to_string(),
            json!({
                "user_id": mkt_ctx.user_id,
                "site_url": mkt_ctx.site_url,
                "total_plugins": mkt_ctx.total_plugins,
                "total_skills": mkt_ctx.total_skills,
                "agents_count": mkt_ctx.agents_count,
                "mcp_count": mkt_ctx.mcp_count,
                "git_url": git_url,
                "cowork_git_url": cowork_git_url,
                "install_cmd": install_cmd,
                "mcp_url": mcp_url,
                "tier_name": mkt_ctx.tier_name,
                "is_premium": mkt_ctx.is_premium,
                "rank_level": mkt_ctx.rank_level,
                "rank_name": mkt_ctx.rank_name,
                "rank_tier": mkt_ctx.rank_tier,
                "total_xp": mkt_ctx.total_xp,
                "xp_progress_pct": numeric::round_to_i64(mkt_ctx.xp_progress_pct),
                "has_completed_onboarding": mkt_ctx.has_completed_onboarding,
                "current_streak": mkt_ctx.current_streak,
                "longest_streak": mkt_ctx.longest_streak,
                "next_rank_name": mkt_ctx.next_rank_name,
                "xp_to_next_rank": mkt_ctx.xp_to_next_rank,
                "plugin_token": mkt_ctx.plugin_token,
            }),
        );
        obj.entry("page_stats".to_string())
            .or_insert_with(|| json!([]));
        if let Some(branding) = engine.branding() {
            if let Ok(val) = serde_json::to_value(branding) {
                obj.insert("branding".to_string(), val);
            }
        }
        if let Some(page_str) = obj.get("page").and_then(|v| v.as_str()) {
            if let Some((help, doc_slug)) = demo_help_text(page_str) {
                obj.insert("demo_help".to_string(), json!(help));
                obj.insert(
                    "demo_help_url".to_string(),
                    json!(format!("/documentation/{}", doc_slug)),
                );
            }
        }
    }
    match engine.render(template, &merged) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(template, error = ?e, "SSR render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Template Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub(crate) fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    super::shared::get_services_path()
}
