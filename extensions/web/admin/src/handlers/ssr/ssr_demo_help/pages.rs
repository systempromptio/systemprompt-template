//! Static help copy keyed by page id.

pub(super) fn demo_help_governance_pages(page: &str) -> Option<(&'static str, &'static str)> {
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

pub(super) fn demo_help_analytics_pages(page: &str) -> Option<(&'static str, &'static str)> {
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

pub(super) fn demo_help_infra_pages(page: &str) -> Option<(&'static str, &'static str)> {
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

pub(super) fn demo_help_entity_edit_pages(page: &str) -> Option<(&'static str, &'static str)> {
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

pub(super) fn demo_help_user_pages(page: &str) -> Option<(&'static str, &'static str)> {
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
        "devices" => Some((
            "<strong>Devices</strong> manages the credentials that the <code>sp-bridge-auth</code> helper uses on your workstation. Issue personal access tokens (PATs) for the PAT flow, view and revoke enrolled device certificates for the mTLS flow, and audit last-used timestamps. The browser-based session flow is handled separately via the per-device consent page.",
            "settings",
        )),
        "bridge-device-link" => Some((
            "<strong>Bridge device consent</strong> is the page the <code>sp-bridge-auth</code> helper opens in your browser when you run the session flow. It shows which local loopback port is requesting credentials, validates the redirect target, and &mdash; on Allow &mdash; mints a 120-second one-shot exchange code the helper trades for a short-lived JWT.",
            "settings",
        )),
        "bridge-setup" => Some((
            "<strong>Connect Claude</strong> walks you through installing the <code>sp-bridge-auth</code> helper and picking one of the three authentication modes (PAT, session, mTLS). The gateway URL is pre-filled, capabilities are queried live, and a one-click copy gives you the exact <code>bridge-auth.toml</code> for this server. You must be signed in to the dashboard before any flow works &mdash; that is the single source of identity.",
            "settings",
        )),
        _ => None,
    }
}

pub(super) fn demo_help_misc_pages(page: &str) -> Option<(&'static str, &'static str)> {
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
