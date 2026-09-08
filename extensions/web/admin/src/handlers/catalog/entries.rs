//! The pieces the plugin and skill listings share.
//!
//! Both pages answer the same four questions about a catalog entry — is it on,
//! what does it carry, who can see it, and what names it directly — so the
//! column set and the tile constructor live here once. The MCP listing does
//! not share them: it has a runtime half these two do not.


use crate::handlers::catalog::sorting::SortColumn;

use super::view::CatalogKpiView;

pub(super) const fn kpi(
    label: &'static str,
    value: String,
    sub: String,
    tone: &'static str,
) -> CatalogKpiView {
    CatalogKpiView {
        label,
        value,
        sub,
        tone,
    }
}

// Why: the entry columns two of the three catalog lists share. Sorting is on
// the assembled rows rather than in a query because the catalog is read from
// YAML on disk — there is no query to push an ORDER BY into.
pub(super) fn entry_columns(
    members_label: &'static str,
    members_hint: &'static str,
) -> Vec<SortColumn> {
    vec![
        SortColumn {
            key: "name",
            label: "Name",
            class: "",
            hint: "The id as declared under services/",
        },
        SortColumn {
            key: "status",
            label: "Status",
            class: "",
            hint: "Whether the entry is enabled in its YAML",
        },
        SortColumn {
            key: "members",
            label: members_label,
            class: "sp-table__cell--num",
            hint: members_hint,
        },
        SortColumn {
            key: "visibility",
            label: "Visibility",
            class: "",
            hint: "Public, or restricted to the roles and groups a rule names",
        },
        SortColumn {
            key: "grants",
            label: "Grants",
            class: "sp-table__cell--num",
            hint: "Access-control rules naming this entry directly",
        },
    ]
}


// Why: The six figures above the plugin listing.
//
// "Public" and "No direct grant" are the two that matter when they are wrong:
// a public plugin is one no rule restricts, and a plugin with no direct grant
// is reachable only through whatever marketplace happens to ship it.
pub(super) fn plugin_kpis(plugins: &[super::view::PluginListRow]) -> Vec<CatalogKpiView> {
    let enabled = plugins.iter().filter(|p| p.enabled).count();
    let public = plugins.iter().filter(|p| p.visibility.is_public).count();
    let skills: usize = plugins.iter().map(|p| p.skills_count).sum();
    let servers: usize = plugins.iter().map(|p| p.mcp_count).sum();
    let ungranted = plugins.iter().filter(|p| p.assignment_count == 0).count();
    vec![
        kpi(
            "Plugins",
            plugins.len().to_string(),
            format!("{enabled} enabled"),
            "",
        ),
        kpi(
            "Skills carried",
            skills.to_string(),
            "across all plugins".to_owned(),
            "",
        ),
        kpi(
            "MCP servers carried",
            servers.to_string(),
            "across all plugins".to_owned(),
            "",
        ),
        kpi(
            "Public",
            public.to_string(),
            "no rule restricts them".to_owned(),
            if public > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "No direct grant",
            ungranted.to_string(),
            "reachable only via a marketplace".to_owned(),
            "",
        ),
        kpi(
            "Disabled",
            (plugins.len() - enabled).to_string(),
            "declared but switched off".to_owned(),
            "",
        ),
    ]
}

// Why: The six figures above the skill listing.
//
// "In no plugin" is the one an operator acts on: a skill nothing ships is
// declared and unreachable, and it is invisible on every usage screen because
// it can never be invoked.
pub(super) fn skill_kpis(
    skills: &[super::view::SkillListRow],
    marketplaces: usize,
) -> Vec<CatalogKpiView> {
    let enabled = skills.iter().filter(|s| s.enabled).count();
    let public = skills.iter().filter(|s| s.visibility.is_public).count();
    let orphaned = skills.iter().filter(|s| s.plugin_count == 0).count();
    let ungranted = skills.iter().filter(|s| s.assignment_count == 0).count();
    vec![
        kpi(
            "Skills",
            skills.len().to_string(),
            format!("{enabled} enabled"),
            "",
        ),
        kpi(
            "In no plugin",
            orphaned.to_string(),
            "declared but nothing ships them".to_owned(),
            if orphaned > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "Public",
            public.to_string(),
            "no rule restricts them".to_owned(),
            if public > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "No direct grant",
            ungranted.to_string(),
            "reachable only via a plugin".to_owned(),
            "",
        ),
        kpi(
            "Disabled",
            (skills.len() - enabled).to_string(),
            "declared but switched off".to_owned(),
            "",
        ),
        kpi(
            "Marketplaces",
            marketplaces.to_string(),
            "that can carry them".to_owned(),
            "",
        ),
    ]
}
