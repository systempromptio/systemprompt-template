//! Which shipped documentation page, if any, a console page's help icon opens.
//!
//! The help table in `handlers::ssr::ssr_demo_help` names a topic per page —
//! `tool-governance`, `mcp-servers`, `gamification`. Those were slugs of a
//! documentation set this instance no longer ships, and the icon linked
//! `/documentation/<topic>` straight through, so 63 of the 73 console pages
//! offered a help link that 404ed. Only `dashboard` happened to survive.
//!
//! Topics are therefore keys, not slugs, and this is the one place they are
//! translated into a page that exists. `None` is a real answer: a topic with no
//! documentation behind it hides the icon rather than promising a page. The
//! unit suite walks the help table and fails if a topic is missing from here or
//! names a file that is not in `services/content/documentation/`.

// Why: every topic the help table emits, against the documentation this
// instance ships. `None` means "written about nowhere yet" — the icon is then
// not drawn, which is honest, where a link to a missing page is not.
#[must_use]
pub const fn doc_slug_for(topic: &str) -> Option<&'static str> {
    Some(match topic.as_bytes() {
        b"dashboard" => "dashboard",
        b"getting-started" | b"my-workspace" => "index",
        b"integration-claude-code" => "connect-claude-code",
        b"access-control" | b"users" => "enterprise-user-access",
        b"activity-tracking" => "enterprise-analytics",
        b"events" => "enterprise-audit-observability",
        b"conversations" => "enterprise-conversation-history",
        b"costs" => "enterprise-cost-management",
        b"models" => "enterprise-model-routing",
        b"safety" => "enterprise-safety-guardrails",
        b"tool-governance" | b"hooks" | b"mcp-servers" | b"marketplace" | b"plugins"
        | b"skills" | b"browse-plugins" => "enterprise-tool-governance",
        b"knowledge" => "enterprise-knowledge-rag",
        b"authentication" | b"secrets" => "authentication",
        b"downloads" => "downloads",
        // Why: no page covers these yet. Listed rather than left to a catch-all
        // so that adding documentation is a change here and not a discovery.
        b"achievements" | b"gamification" | b"agents" | b"jobs" | b"profile" | b"settings" => {
            return None;
        },
        _ => return None,
    })
}

// Why: the URL the console links, or nothing. Kept beside the table so a caller
// cannot build the path from a topic that has no page.
#[must_use]
pub fn documentation_url(topic: &str) -> Option<String> {
    doc_slug_for(topic).map(|slug| format!("/documentation/{slug}"))
}
