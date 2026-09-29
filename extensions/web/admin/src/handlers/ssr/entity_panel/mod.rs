//! The "Who gets this" panel every catalog detail page carries.
//!
//! Only the link to an entity's own panel lives here so far — the access
//! review on `/admin/sync` points each row at it. The panel builder, its
//! partial (`components/entity-access`) and its script land with the access
//! review port (Stage 3 phase 5), which replaces this file with the full
//! module.

// Why: the page every "access" link lands on — the entity's own panel. A
// kind with no catalog detail page (a gateway route, say) has none.
#[must_use]
pub(crate) fn entity_access_url(entity_type: &str, entity_id: &str) -> Option<String> {
    let base = match entity_type {
        "marketplace" => "/admin/marketplaces",
        "plugin" => "/admin/plugins",
        "skill" => "/admin/skills",
        "mcp_server" => "/admin/mcp",
        "gateway_route" => "/admin/gateway/routes",
        _ => return None,
    };
    Some(format!("{base}/{}#access", urlencoding::encode(entity_id)))
}
