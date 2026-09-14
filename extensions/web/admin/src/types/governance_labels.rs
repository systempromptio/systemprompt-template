//! How a governance decision names itself: which plane a policy belongs to, and
//! what a row was actually about.
//!
//! Both were derivations inside the governance page's view module, where the
//! tests workspace could not reach them — and both had failed silently for
//! months precisely because nothing could assert on them. They are pure string
//! functions over values the database hands back, so they live here, beside the
//! other display types, and the unit suite pins them.

// Why: where a policy comes from, for every policy that is actually written.
// The predecessor knew only the four synchronous chain stages and answered an
// em-dash for anything else, which on an instance whose traffic is `authz`,
// `default_allow` and `authentication` meant an em-dash on every row.
//
// The final arm is the load-bearing one: an unrecognised policy names itself,
// so a producer nobody knew about announces its arrival instead of vanishing
// into a dash.
#[must_use]
pub fn plane_of(policy: &str) -> String {
    let known = match policy {
        "agent_scope" => "chain · 1 scope",
        "secret_scan" => "chain · 2 secret",
        "tool_blocklist" => "chain · 3 blocklist",
        "rate_limit" => "chain · 4 rate",
        "default_allow" => "chain · pass",
        "governance_allow" => "chain · allow rule",
        "governance_disabled" => "chain · off",
        "quota" => "gateway · quota",
        "authentication" => "gateway · authentication",
        "authz" => "authz · extension",
        "authz_extension_hook" => "authz · hook",
        "authz_rule_based" => "authz · rules",
        "authz_default_deny" => "authz · default deny",
        "authz_unrestricted" => "authz · unrestricted",
        "authz_hook_fault" => "authz · fault",
        other => return other.to_owned(),
    };
    known.to_owned()
}

// Why: what a decision was about, from the overloaded `tool_name` column.
// That column carries three different things depending on which producer wrote
// the row: the gateway gate's `user_prompt` sentinel, a real MCP tool name, or
// an authorization entity id whose kind lives in `evaluated_rules`. Rendered
// raw, `user_prompt` and `claude-star-4203d1` sat under a heading that said
// "Tool" and neither of them was one.
#[must_use]
pub fn target_label(tool_name: &str, entity_type: Option<&str>) -> String {
    if let Some(kind) = entity_type.filter(|k| !k.is_empty()) {
        return format!("{kind} {tool_name}");
    }
    if tool_name == "user_prompt" {
        return "prompt".to_owned();
    }
    if tool_name.is_empty() {
        return "\u{2014}".to_owned();
    }
    format!("tool {tool_name}")
}
