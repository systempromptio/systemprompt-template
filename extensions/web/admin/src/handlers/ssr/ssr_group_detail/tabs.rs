//! The group detail tab strip.
//!
//! Tabs are links and the server renders only the active tab's body, so a
//! view is bookmarkable and the page never ships five tabs' worth of queries
//! to show one.

use super::super::types::TabLinkView;

pub(super) const USAGE: &str = "usage";
pub(super) const MEMBERS: &str = "members";
pub(super) const MARKETPLACES: &str = "marketplaces";
pub(super) const ACCESS: &str = "access";
pub(super) const PROJECTS: &str = "projects";
pub(super) const MAPPINGS: &str = "mappings";

pub(super) struct TabCounts {
    pub members: i64,
    pub marketplaces: i64,
    pub projects: i64,
    pub mappings: i64,
}

// Why: Why: the Unassigned bucket has no rules and no directory mapping of its
// own — it is where people land when no mapping matched — so those two tabs
// would render an editor for a subject that can never hold a rule.
pub(super) const fn visible_tabs(is_unassigned: bool) -> &'static [&'static str] {
    if is_unassigned {
        &[MEMBERS, USAGE, PROJECTS]
    } else {
        &[USAGE, MEMBERS, MARKETPLACES, PROJECTS, MAPPINGS, ACCESS]
    }
}

pub(super) const fn default_tab(is_unassigned: bool) -> &'static str {
    if is_unassigned { MEMBERS } else { USAGE }
}

pub(super) fn resolve_tab(requested: Option<&str>, is_unassigned: bool) -> &'static str {
    let visible = visible_tabs(is_unassigned);
    requested
        .and_then(|r| visible.iter().find(|t| **t == r).copied())
        .unwrap_or_else(|| default_tab(is_unassigned))
}

pub(super) fn tab_links(
    group_id: &str,
    active: &str,
    is_unassigned: bool,
    counts: &TabCounts,
) -> Vec<TabLinkView> {
    visible_tabs(is_unassigned)
        .iter()
        .map(|&slug| TabLinkView {
            slug,
            label: label_for(slug),
            href: format!("/admin/groups/{group_id}?tab={slug}"),
            is_active: slug == active,
            count: count_for(slug, counts),
        })
        .collect()
}

fn label_for(slug: &str) -> &'static str {
    match slug {
        MEMBERS => "Members",
        MARKETPLACES => "Marketplaces",
        ACCESS => "Access",
        PROJECTS => "Projects",
        MAPPINGS => "Directory mappings",
        _ => "Usage",
    }
}

fn count_for(slug: &str, counts: &TabCounts) -> Option<i64> {
    match slug {
        MEMBERS => Some(counts.members),
        MARKETPLACES => Some(counts.marketplaces),
        PROJECTS => Some(counts.projects),
        MAPPINGS => Some(counts.mappings),
        _ => None,
    }
}
