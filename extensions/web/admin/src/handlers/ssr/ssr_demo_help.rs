pub fn demo_help_text(page: &str) -> Option<(&'static str, &'static str)> {
    Some(
        demo_help_core_pages(page)
            .or_else(|| demo_help_admin_pages(page))
            .or_else(|| demo_help_governance_pages(page))
            .or_else(|| demo_help_analytics_pages(page))
            .or_else(|| demo_help_infra_pages(page))
            .or_else(|| demo_help_entity_edit_pages(page))
            .or_else(|| demo_help_user_pages(page))
            .or_else(|| demo_help_misc_pages(page))
            .unwrap_or((
                "This page is part of the <strong>Enterprise Demo</strong>. It exercises one slice of the governance, analytics, or workspace pipeline end-to-end so you can see how the platform behaves in production. Data shown here is real telemetry from the running stack &mdash; connect via <strong>Claude Code</strong> to populate it with your own sessions. Claude Code is the recommended integration while Cowork (research preview) stabilises.",
                "dashboard",
            )),
    )
}

fn demo_help_core_pages(page: &str) -> Option<(&'static str, &'static str)> {
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
        _ => None,
    }
}

fn demo_help_admin_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
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

fn demo_help_governance_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "governance-audit" => Some((
            "<strong>Governance Audit Trail</strong> is the immutable record of every policy decision the platform has made &mdash; each row shows which tool call was evaluated, which rule fired, whether it was allowed or denied, and why. Use this page to prove compliance, investigate incidents, and reconstruct what Claude did during any session. In production the audit log is the system of record your security and compliance teams rely on.",
            "tool-governance",
        )),
        "governance-decisions" => Some((
            "<strong>Governance Decisions</strong> streams the live feed of allow/deny/modify verdicts produced by the policy engine as Claude invokes tools. Each decision is linked back to the matching rule, the user, the session, and the cost impact. Use it to spot-check policy behaviour in real time and tune rules before they reach production traffic.",
            "tool-governance",
        )),
        "governance-violations" => Some((
            "<strong>Governance Violations</strong> lists the tool calls that were denied or flagged by policy. Each entry shows the offending rule, the user and role, the affected session, and the remediation status. This is the workbench security teams use to triage incidents, open follow-ups, and decide whether a rule needs tightening or an exception needs granting.",
            "tool-governance",
        )),
        "governance-rules" => Some((
            "<strong>Governance Rules</strong> is the policy editor &mdash; define the allow/deny/modify rules that gate every tool call Claude makes. Rules are scoped by role, department, tool, and conditions; they run on the hot path of every request and are enforced at the MCP layer. Edit here to change what your team is allowed to do with Claude.",
            "tool-governance",
        )),
        "governance-hooks" => Some((
            "<strong>Governance Hooks</strong> configures the lifecycle hooks that fire during Claude Code sessions &mdash; PreToolUse, PostToolUse, SessionStart, PermissionRequest, and more. Hooks let you inject compliance logic, audit writes, automated approvals, or content filtering at each stage without changing application code. Hooks work best with <strong>Claude Code</strong>, the recommended integration.",
            "hooks",
        )),
        "governance-rate-limits" => Some((
            "<strong>Governance Rate Limits</strong> defines and monitors usage quotas per user, role, department, tool, and time window. Rate limits protect budgets, enforce fair-use across teams, and prevent runaway automation. This page shows current consumption against configured ceilings and lets you adjust limits as usage patterns evolve.",
            "tool-governance",
        )),
        _ => None,
    }
}

fn demo_help_analytics_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "analytics-overview" => Some((
            "<strong>Analytics Overview</strong> is the executive summary of Claude usage across your organisation &mdash; total sessions, active users, tool call volume, cost trend, and top performing plugins. Use it to spot week-over-week changes at a glance before drilling into the per-domain analytics pages. All numbers are live, computed from the same telemetry that drives the governance pipeline.",
            "dashboard",
        )),
        "analytics-agents" => Some((
            "<strong>Analytics &mdash; Agents</strong> ranks every agent configuration by session count, effectiveness rating, tool-call success rate, and average cost per session. Use it to find which agents your team actually uses, which ones underperform, and which are worth promoting or retiring.",
            "agents",
        )),
        "analytics-content" => Some((
            "<strong>Analytics &mdash; Content</strong> measures how published skills, guides, and marketplace pages perform &mdash; traffic, engagement, search rank, and conversion. It ties content output to actual platform adoption so you can see which documentation is load-bearing and which is dead weight.",
            "dashboard",
        )),
        "analytics-conversations" => Some((
            "<strong>Analytics &mdash; Conversations</strong> aggregates every Claude conversation into trends: topics, session length, tool-call density, hand-off points, and user satisfaction signals. Use it to understand what your team is actually asking Claude and where the friction lives.",
            "events",
        )),
        "analytics-costs" => Some((
            "<strong>Analytics &mdash; Costs</strong> breaks Claude spend down by user, department, plugin, model, and tool. See exactly where token spend lands, catch runaway agents early, and allocate costs back to the teams that incurred them. The same data feeds governance rate-limit decisions.",
            "dashboard",
        )),
        "analytics-requests" => Some((
            "<strong>Analytics &mdash; Requests</strong> lists every AI request the platform has processed &mdash; prompt, response, tools invoked, latency, token count, and cost. Drill into any request to get the full conversation context. This is the primary surface for debugging individual AI failures in production.",
            "events",
        )),
        "analytics-sessions" => Some((
            "<strong>Analytics &mdash; Sessions</strong> shows the session-level view of Claude usage: who opened what, how long it ran, which tools it touched, and how it ended. Use it to reconstruct user journeys and spot session patterns that simple request logs hide.",
            "events",
        )),
        "analytics-tools" => Some((
            "<strong>Analytics &mdash; Tools</strong> ranks every MCP tool by invocation count, success rate, average latency, and governance verdict distribution. Use it to find hot tools, failing tools, and tools that governance keeps denying &mdash; the signals you need to tune your MCP layer.",
            "mcp-servers",
        )),
        _ => None,
    }
}

fn demo_help_infra_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "infra-config" => Some((
            "<strong>Infra &mdash; Config</strong> is the live view of the merged <code>services/</code> configuration tree &mdash; agents, skills, MCP servers, plugins, AI providers, and scheduler. Inspect what the running process actually loaded, including values resolved from includes. Edits happen in the YAML files under <code>services/</code>; this page is the read-only source of truth for &ldquo;what is currently live?&rdquo;.",
            "dashboard",
        )),
        "infra-database" => Some((
            "<strong>Infra &mdash; Database</strong> provides a guarded query console over the platform's operational database. Run ad-hoc SELECTs to inspect events, sessions, governance decisions, users, and usage counters without leaving the admin UI. In production this page is gated behind the infra-admin role.",
            "dashboard",
        )),
        "infra-logs" => Some((
            "<strong>Infra &mdash; Logs</strong> aggregates the platform log streams &mdash; application, MCP servers, governance, and jobs &mdash; with level filtering and time-range controls. It is the first stop when debugging a failing session, a failed job, or an MCP tool that is misbehaving. The same data is available from the CLI via <code>systemprompt infra logs</code>.",
            "dashboard",
        )),
        "infra-services" => Some((
            "<strong>Infra &mdash; Services</strong> shows every runtime service the platform manages &mdash; AI providers, MCP servers, schedulers, web frontends, content sources &mdash; with live health, uptime, and restart controls. Use it to verify what is running, diagnose startup failures, and restart misbehaving components.",
            "dashboard",
        )),
        _ => None,
    }
}

fn demo_help_entity_edit_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "skill-edit" => Some((
            "<strong>Skill Editor</strong> lets you author and tune an individual skill &mdash; the prompt template, instructions, metadata, and bound content. Changes save into the skill definition and are picked up by any plugin that references it. Use this page to refine how Claude handles a specific task without touching any code.",
            "skills",
        )),
        "skills-content" => Some((
            "<strong>Skill Content</strong> manages the reference material attached to a skill &mdash; examples, source documents, and context the skill injects into Claude's prompt. Keep it tight: content bloat directly inflates every session's token cost.",
            "skills",
        )),
        "skills-contexts" => Some((
            "<strong>Skill Contexts</strong> defines the runtime contexts in which a skill is offered &mdash; which agents, which plugins, which triggers. Use it to control when Claude should reach for this skill versus another.",
            "skills",
        )),
        "skills-files" => Some((
            "<strong>Skill Files</strong> manages the attached files a skill ships with &mdash; templates, config, static assets. Files are versioned alongside the skill definition and distributed with any plugin that bundles the skill.",
            "skills",
        )),
        "skills-plugins" => Some((
            "<strong>Skill Plugins</strong> shows which plugins currently include this skill. Use it to understand blast radius before editing: any change here ripples into every plugin listed.",
            "skills",
        )),
        "agent-edit" | "my-agent-edit" => Some((
            "<strong>Agent Editor</strong> configures a single agent &mdash; system prompt, assigned skills, tool access, model, and guardrails. Each change is versioned and takes effect on the next Claude session that invokes the agent. Use this page to tune agent behaviour against real usage data.",
            "agents",
        )),
        "agent-config" => Some((
            "<strong>Agent Config</strong> is the structured configuration panel for an agent &mdash; model settings, context window, sampling parameters, and advanced controls. Edit here for behaviour tweaks that don't belong in the prompt itself.",
            "agents",
        )),
        "agent-messages" => Some((
            "<strong>Agent Messages</strong> shows the conversation history produced by this agent &mdash; the prompts it received, the tools it called, and the responses it returned. Use it to audit agent behaviour and debug regressions after a prompt change.",
            "agents",
        )),
        "agent-traces" => Some((
            "<strong>Agent Traces</strong> is the per-agent trace viewer: every session this agent ran, with tool-call timeline, governance verdicts, latency, and cost. The fastest way to see whether an agent is healthy in production.",
            "events",
        )),
        "mcp-access" => Some((
            "<strong>MCP Access</strong> controls which users, roles, and plugins can reach each MCP server. Access is enforced at invocation time &mdash; when Claude tries to call a tool, the platform checks the caller's permissions before the request leaves the process.",
            "mcp-servers",
        )),
        "mcp-tools" => Some((
            "<strong>MCP Tools</strong> lists every tool exposed by the connected MCP servers with its schema, last-call status, and governance verdicts. Use it to discover what Claude can actually do in your workspace and to validate that tool definitions match what you expect.",
            "mcp-servers",
        )),
        "mcp-edit" => Some((
            "<strong>MCP Editor</strong> configures a single Model Context Protocol server &mdash; transport, command, environment, and auth. Changes here reload the server in-place. Use it to wire up new integrations or fix misbehaving ones without leaving the admin UI.",
            "mcp-servers",
        )),
        "my-plugin-edit" => Some((
            "<strong>Plugin Editor</strong> is where you customise an installed plugin &mdash; fork it, edit its skills, swap agents, rewire MCP servers, and publish the result to your marketplace. Forked plugins stay linked to upstream so you can pull updates selectively.",
            "plugins",
        )),
        "my-plugin-view" => Some((
            "<strong>Plugin Detail</strong> shows everything bundled into a single plugin &mdash; skills, agents, MCP servers, hooks, and secrets &mdash; along with install stats and version history. Use it to audit exactly what an installed plugin brings into your workspace.",
            "plugins",
        )),
        "hook-edit" => Some((
            "<strong>Hook Editor</strong> configures a single hook &mdash; which lifecycle event it listens on, the matcher that scopes it, and the action it runs. Hooks are the extension point for compliance, analytics, and custom approval flows. They fire natively in <strong>Claude Code</strong> sessions.",
            "hooks",
        )),
        _ => None,
    }
}

fn demo_help_user_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "user-detail" => Some((
            "<strong>User Detail</strong> is the per-user admin view &mdash; profile, roles, departments, session history, tool usage, governance violations, cost attribution, and activity timeline. The single place to investigate what any individual has been doing on the platform.",
            "users",
        )),
        "users-sessions" => Some((
            "<strong>User Sessions</strong> lists every active and recent authentication session across the platform &mdash; user, IP, device, issue time, last seen, and revoke controls. Use it to audit account access and force logouts when needed.",
            "users",
        )),
        "users-ip-bans" => Some((
            "<strong>IP Bans</strong> manages the platform's IP-level blocklist. Add, remove, and inspect banned addresses with reason and ban timestamp. Bans are enforced at the HTTP edge before any auth logic runs.",
            "users",
        )),
        _ => None,
    }
}

fn demo_help_misc_pages(page: &str) -> Option<(&'static str, &'static str)> {
    match page {
        "perf-benchmarks" => Some((
            "<strong>Performance Benchmarks</strong> shows the platform's internal latency and throughput measurements &mdash; request paths, MCP tool round-trips, governance evaluation time, and database query budgets. Use it to catch regressions before they hit users.",
            "dashboard",
        )),
        "perf-traces" => Some((
            "<strong>Performance Traces</strong> is the distributed-trace viewer for the platform itself: every span of a request, with timing and context. Use it to diagnose slow pages, slow tool calls, and slow jobs end-to-end.",
            "events",
        )),
        "demo-register" => Some((
            "<strong>Demo Registration</strong> is the lightweight sign-up flow for the Enterprise Demo. It provisions a temporary workspace with pre-seeded plugins, agents, and sample telemetry so you can evaluate the full governance pipeline end-to-end without any setup overhead.",
            "getting-started",
        )),
        _ => None,
    }
}
