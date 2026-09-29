//! One skill row and the marketplace → plugin grouping of the rows.

use std::collections::{BTreeMap, HashMap};

use chrono::{Duration, NaiveDate};
use systemprompt::identifiers::MarketplaceId;

use super::SPARK_DAYS;
use super::views::{MarketplaceGroupView, PluginGroupView, SkillFactView};
use crate::handlers::ssr::analysis::tone::{
    completion_tone, deny_tone, error_rate_tone, latency_tone, percent, score_display,
};
use crate::handlers::ssr::analysis_urls::{analysis_skill_url, analysis_version_url};
use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::handlers::ssr::types::sparkline_toned;
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::SkillFactRow;

// Why: the row's `spark_days` are only the days with an invocation; the
// sparkline wants every one of the last fourteen, zeros included.
pub(super) fn spark_values(row: &SkillFactRow, end: NaiveDate) -> Vec<i64> {
    let by_day: HashMap<NaiveDate, i64> = row
        .spark_days
        .iter()
        .copied()
        .zip(row.spark_counts.iter().copied())
        .collect();
    (0..SPARK_DAYS)
        .rev()
        .map(|back| {
            by_day
                .get(&(end - Duration::days(back)))
                .copied()
                .unwrap_or(0)
        })
        .collect()
}

// Why: the Tools / Artifacts pages default to the last 24 hours; a link from
// a skill row spans the skill's own use in this window.
fn activity_href(base: &str, skill: &str, first_used: chrono::DateTime<chrono::Utc>) -> String {
    let from = first_used - Duration::hours(1);
    format!(
        "{base}?skill={}&from={}&to={}",
        urlencoding::encode(skill),
        urlencoding::encode(&from.to_rfc3339()),
        urlencoding::encode(&chrono::Utc::now().to_rfc3339())
    )
}

fn reach_display(users: i64, entitled: usize) -> String {
    if entitled == 0 {
        return "—".to_owned();
    }
    format!(
        "{}%",
        users * 100 / i64::try_from(entitled).unwrap_or(1).max(1)
    )
}

pub(crate) fn skill_row_view(
    row: &SkillFactRow,
    audience: &MarketplaceAudience,
    index: &InventoryIndex,
    today: NaiveDate,
) -> SkillFactView {
    let skill_name = row
        .skill
        .split_once(':')
        .map_or_else(|| row.skill.clone(), |(_, s)| s.to_owned());
    let entitled = audience.users_reaching_skill(index, &skill_name.replace('-', "_"));
    let tokens = row.input_tokens + row.output_tokens;
    SkillFactView {
        href: analysis_skill_url(&row.skill),
        skill: row.skill.clone(),
        skill_name,
        plugin: row
            .plugin_id
            .as_ref()
            .map(|p| p.as_str().to_owned())
            .unwrap_or_default(),
        marketplace: row
            .marketplace_id
            .as_ref()
            .map(|m| m.as_str().to_owned())
            .unwrap_or_default(),
        invocations: row.invocations,
        invocations_note: format!("{} slash · {} tool", row.slash, row.tool),
        users: row.users,
        entitled,
        reach_display: reach_display(row.users, entitled),
        installs: row.installs,
        conversations: row.conversations,
        turns: row.turns,
        tokens_display: format_token_total(tokens),
        tokens_title: format!(
            "{} in · {} out · {} cached",
            row.input_tokens, row.output_tokens, row.cache_tokens
        ),
        cost_display: format_cost(row.cost_microdollars),
        cost_per_invocation: format_cost(row.cost_microdollars / row.invocations.max(1)),
        errors: row.errors,
        errors_tone: error_rate_tone(row.errors, row.requests),
        denied: row.denied,
        denied_tone: deny_tone(row.denied),
        tool_calls: row.tool_calls,
        tool_calls_failed: row.tool_calls_failed,
        tools_href: activity_href("/admin/tools", &row.skill, row.first_used),
        artifacts: row.artifacts,
        artifacts_href: activity_href("/admin/artifacts", &row.skill, row.first_used),
        latency_display: row.p95_latency_ms.map_or_else(
            || "—".to_owned(),
            |ms| format_duration_ms(ms.round() as i64),
        ),
        latency_tone: latency_tone(row.p95_latency_ms),
        models: row.models.clone(),
        clients: row.clients.clone(),
        judged: row.judged,
        completion_display: score_display(row.completion_avg),
        completion_tone: completion_tone(row.completion_avg),
        completion_note: format!("{}/{}", row.judged, row.conversations),
        attributed_pct: percent(row.attributed, row.invocations),
        spark: sparkline_toned(
            &spark_values(row, today),
            "accent",
            format!("Invocations per day, last {SPARK_DAYS} days"),
        ),
        first_used: row.first_used.format("%b %-d").to_string(),
        last_used: row.last_used.format("%b %-d, %H:%M").to_string(),
    }
}

pub(super) fn marketplace_name(index: &InventoryIndex, id: &str) -> String {
    index
        .marketplaces
        .iter()
        .find(|m| m.id.as_str() == id)
        .map_or_else(|| id.to_owned(), |m| m.name.clone())
}

pub(super) fn group_rows(
    rows: Vec<SkillFactView>,
    index: &InventoryIndex,
) -> Vec<MarketplaceGroupView> {
    let mut by_market: BTreeMap<String, BTreeMap<String, Vec<SkillFactView>>> = BTreeMap::new();
    for row in rows {
        by_market
            .entry(row.marketplace.clone())
            .or_default()
            .entry(row.plugin.clone())
            .or_default()
            .push(row);
    }
    let mut groups: Vec<MarketplaceGroupView> = by_market
        .into_iter()
        .map(|(marketplace, plugins)| {
            let plugins: Vec<PluginGroupView> = plugins
                .into_iter()
                .map(|(plugin, rows)| PluginGroupView {
                    invocations: rows.iter().map(|r| r.invocations).sum(),
                    plugin,
                    rows,
                })
                .collect();
            let invocations = plugins.iter().map(|p| p.invocations).sum();
            let skills = plugins.iter().map(|p| p.rows.len()).sum();
            let users = plugins
                .iter()
                .flat_map(|p| p.rows.iter().map(|r| r.users))
                .max()
                .unwrap_or(0);
            MarketplaceGroupView {
                marketplace_name: if marketplace.is_empty() {
                    "Unplaced".to_owned()
                } else {
                    marketplace_name(index, &marketplace)
                },
                versions_href: analysis_version_url(&MarketplaceId::new(&marketplace), None),
                marketplace,
                plugins,
                skills,
                invocations,
                users,
            }
        })
        .collect();
    groups.sort_by_key(|g| std::cmp::Reverse(g.invocations));
    groups
}
