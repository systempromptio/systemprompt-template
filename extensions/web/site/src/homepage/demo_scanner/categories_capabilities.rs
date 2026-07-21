//! Static copy for the capability-oriented demo categories.

use super::meta::CategoryMeta;

pub(super) const CAPABILITY_CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "governance",
        title: "Governance",
        tagline: "Policy that runs on every tool call, not a slide in a security review.",
        story: "Staff engineers want to know what actually happens when an agent goes off-script. This walkthrough fires real PreToolUse hooks against the governance endpoint: a clean admin call passes all three rules, a user-scope agent gets denied at the scope and blocklist layers, and plaintext AWS keys, GitHub PATs, and PEM private keys are blocked before they reach a tool. Every decision lands in an audit table you can query, capped by rate limits you can inspect and hooks you can extend.",
        cost: "",
        feature_url: "https://systemprompt.io/features/governance-pipeline",
    },
    CategoryMeta {
        id: "agents",
        title: "Agents",
        tagline: "From \"which agents ship with the platform?\" to a fully traced AI call in five commands.",
        story: "Staff engineers evaluating an agent runtime want three things: to see what's configured, to watch one actually do work, and to trust that every call is auditable. This walkthrough starts at agent discovery, drills into the config and tool scopes that separate an admin agent from a user-scoped one, sends a live message that triggers real MCP tool use, and ends at the execution trace and A2A registry. By the last step you have seen events, artifacts, cost attribution, and service discovery for a single prompt.",
        cost: "API key required",
        feature_url: "https://systemprompt.io/features/closed-loop-agents",
    },
    CategoryMeta {
        id: "mcp",
        title: "MCP Servers",
        tagline: "Every MCP tool call is inventoried, governed, and costed — watch it catch a leaking secret.",
        story: "MCP is where agents meet real systems, which is exactly where enterprise reviews stall: who is connected, who called what, and what stops a misbehaving model from exfiltrating a secret. This walkthrough starts at the server inventory, then fires a governance hook with a clean payload and a payload containing an AWS key to show allow and deny decisions hitting the audit tables, and ends in the execution analytics view where every tool call is already counted.",
        cost: "",
        feature_url: "https://systemprompt.io/features/mcp-governance",
    },
    CategoryMeta {
        id: "analytics",
        title: "Analytics",
        tagline: "Every agent call, every token, every dollar — queryable from the CLI.",
        story: "If you can't answer \"what did my agents do today and what did it cost?\" you can't run them in production. This walkthrough starts at the 24-hour overview and drills down: which agents are busy, what each model is costing, and the raw request stream with latency and token counts. The same telemetry powers the dashboard, so nothing here is a toy view.",
        cost: "",
        feature_url: "https://systemprompt.io/features/analytics-and-observability",
    },
    CategoryMeta {
        id: "users",
        title: "Users & Access",
        tagline: "Identity, roles, sessions, and abuse response from one CLI.",
        story: "Before trusting a platform with production agents, you want to know how user identity actually works. This walkthrough lists the user directory, drills into a single user's role, confirms the current authenticated session and profile, then demonstrates the IP-ban lever end to end — add, verify, remove — so you know the abuse-response path is real and reversible.",
        cost: "",
        feature_url: "https://systemprompt.io/features/compliance",
    },
];
