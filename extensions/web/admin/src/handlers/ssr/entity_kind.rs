//! The one spelling of each governed entity kind the console shows.
//!
//! `EntityKind::as_str()` is the wire form (`gateway_route`); pages used to
//! derive a label by swapping underscores for spaces, which gave "gateway
//! route" — true, but not what an operator calls the thing. A model route is
//! a model route.

#[must_use]
pub(crate) fn entity_kind_label(kind: &str) -> &'static str {
    match kind {
        "gateway_route" => "Model route",
        "mcp_server" => "MCP server",
        "marketplace" => "Marketplace",
        "plugin" => "Plugin",
        "skill" => "Skill",
        "agent" => "Agent",
        "hook" => "Hook",
        "slack_workspace" => "Slack workspace",
        "slack_channel" => "Slack channel",
        "teams_tenant" => "Teams tenant",
        "teams_conversation" => "Teams conversation",
        _ => "Entity",
    }
}

// Why: Plural form for section headings and group rows.
#[must_use]
pub(crate) fn entity_kind_plural(kind: &str) -> &'static str {
    match kind {
        "gateway_route" => "Model routes",
        "mcp_server" => "MCP servers",
        "marketplace" => "Marketplaces",
        "plugin" => "Plugins",
        "skill" => "Skills",
        "agent" => "Agents",
        "hook" => "Hooks",
        _ => "Entities",
    }
}

// Why: Display order for grouped entity listings: the things people install
// first, then what they run through.
#[must_use]
pub(crate) fn entity_kind_rank(kind: &str) -> u8 {
    match kind {
        "marketplace" => 0,
        "plugin" => 1,
        "skill" => 2,
        "mcp_server" => 3,
        "gateway_route" => 4,
        "agent" => 5,
        "hook" => 6,
        _ => 9,
    }
}
