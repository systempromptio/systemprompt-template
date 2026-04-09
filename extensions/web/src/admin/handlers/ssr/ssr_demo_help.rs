pub fn demo_help_text(page: &str) -> Option<(&'static str, &'static str)> {
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
