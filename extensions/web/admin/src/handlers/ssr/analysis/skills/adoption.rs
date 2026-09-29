//! One Overview row per marketplace: entitled → installed → active, with
//! the window's activity and spend. The install rate counts only installed
//! people the rules entitle, so a consumer outside the entitlement is shown
//! beside the count rather than pushing the rate past 100.

use super::rows::marketplace_name;
use super::views::AdoptionView;
use super::{SkillsQuery, SkillsTab};
use crate::handlers::ssr::analysis::tone::{completion_tone, percent, score_display};
use crate::handlers::ssr::analysis_urls::analysis_version_url;
use crate::handlers::ssr::format::format_cost;
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::MarketplaceAdoptionRow;

fn share(part: i64, whole: i64) -> i64 {
    if whole > 0 {
        (part * 100 / whole).min(100)
    } else {
        0
    }
}

pub(super) fn adoption_view(
    query: &SkillsQuery,
    a: &MarketplaceAdoptionRow,
    audience: &MarketplaceAudience,
    index: &InventoryIndex,
) -> AdoptionView {
    let id = a.marketplace_id.as_str();
    let entitled = audience.users_reaching_marketplace(id);
    let entitled_i = i64::try_from(entitled).unwrap_or(0);
    let installed_entitled = audience.entitled_among(id, &a.installed_consumers);
    let installed_entitled_i = i64::try_from(installed_entitled).unwrap_or(0);
    let active_capped = a.active_users.min(a.installed);
    AdoptionView {
        marketplace_name: marketplace_name(index, id),
        versions_href: analysis_version_url(&a.marketplace_id, None),
        skills_href: query.link_marketplace(SkillsTab::Skills, Some(id)),
        activity_href: query.link_marketplace(SkillsTab::Activity, Some(id)),
        marketplace: id.to_owned(),
        entitled,
        installed: a.installed,
        installed_entitled,
        installed_outside: a.installed - installed_entitled_i,
        active: a.active_users,
        install_rate: percent(installed_entitled_i, entitled_i),
        install_rate_pct: share(installed_entitled_i, entitled_i),
        activation_rate: percent(active_capped, a.installed),
        active_pct: share(a.active_users, entitled_i),
        hosts_title: format!(
            "{} claude-code · {} opencode · {} other",
            a.installed_claude_code, a.installed_opencode, a.installed_other
        ),
        skills: a.skills,
        skills_used: a.skills_used,
        plugins: a.plugins,
        invocations: a.invocations,
        conversations: a.conversations,
        cost_display: format_cost(a.cost_microdollars),
        completion_display: score_display(a.completion_avg),
        completion_tone: completion_tone(a.completion_avg),
        version_short: a
            .current_hash
            .as_deref()
            .map_or_else(|| "—".to_owned(), |h| h[..h.len().min(12)].to_owned()),
        versions: a.versions,
        last_install: a
            .last_install_at
            .map_or_else(|| "never".to_owned(), |t| t.format("%b %-d").to_string()),
    }
}
