//! The KPI strip over the marketplace list: the six facts an operator scans
//! before opening any one marketplace.

use super::view::{AudienceMatrixView, MarketplaceCardView, MarketplaceKpiView};

// Why: the six facts an operator scans before opening any one marketplace.
// "Reachable" is the resolved count, so a manifest that grants a group nothing
// can reach reads as zero here rather than as its declared list.
pub(super) fn list_kpis(
    cards: &[MarketplaceCardView],
    audience: &AudienceMatrixView,
) -> Vec<MarketplaceKpiView> {
    let enabled = cards.iter().filter(|c| c.enabled).count();
    let open = cards.iter().filter(|c| c.default_included).count();
    let plugins: usize = cards.iter().map(|c| c.plugin_count).sum();
    let skills: usize = cards.iter().map(|c| c.skill_count).sum();
    let assigned: usize = cards.iter().map(|c| c.assigned_group_count).sum();
    let unreachable = cards.iter().filter(|c| c.allowed_subjects == 0).count();
    let kpi =
        |label: &'static str, value: String, sub: String, tone: &'static str| MarketplaceKpiView {
            label,
            value,
            sub,
            tone,
        };
    vec![
        kpi(
            "Marketplaces",
            cards.len().to_string(),
            format!("{enabled} enabled"),
            "",
        ),
        kpi(
            "Group grants",
            assigned.to_string(),
            format!("across {} subjects", audience.rows.len()),
            "",
        ),
        kpi(
            "Reachable by nobody",
            unreachable.to_string(),
            "no group or role resolves to allow".to_owned(),
            if unreachable > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "Open to the estate",
            open.to_string(),
            "default_included is true".to_owned(),
            if open > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "Plugins",
            plugins.to_string(),
            "declared members".to_owned(),
            "",
        ),
        kpi(
            "Skills",
            skills.to_string(),
            "carried by those plugins".to_owned(),
            "",
        ),
    ]
}
