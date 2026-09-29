//! The Overview's KPI strip and the unused-skills list under the table —
//! each computed over the rows the scope admitted.

use chrono::NaiveDate;

use super::rows::spark_values;
use super::views::{SkillsKpiView, UnusedSkillView};
use crate::handlers::ssr::analysis::tone::{
    completion_tone, error_rate_tone, percent, score_display,
};
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::sparkline_toned;
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::{MarketplaceAdoptionRow, SkillFactRow};

// Why: the scope's headline figures, computed once and read by every tile.
struct Figures {
    spark_sum: Vec<i64>,
    completion: Option<f64>,
    judged: i64,
    installed: i64,
    installed_entitled: i64,
    entitled: i64,
    active: i64,
    invocations: i64,
    cost: i64,
}

impl Figures {
    fn new(
        rows: &[SkillFactRow],
        adoption: &[&MarketplaceAdoptionRow],
        audience: &MarketplaceAudience,
        marketplace: Option<&str>,
        today: NaiveDate,
    ) -> Self {
        let judged_rows: Vec<&SkillFactRow> =
            rows.iter().filter(|r| r.completion_avg.is_some()).collect();
        let judged: i64 = judged_rows.iter().map(|r| r.judged).sum();
        let completion = (judged > 0).then(|| {
            judged_rows
                .iter()
                .map(|r| r.completion_avg.unwrap_or(0.0) * r.judged as f64)
                .sum::<f64>()
                / judged as f64
        });
        let consumers: Vec<String> = adoption
            .iter()
            .flat_map(|a| a.installed_consumers.iter().cloned())
            .collect();
        let (entitled, installed_entitled) = marketplace.map_or_else(
            || {
                (
                    audience.users_reaching_any_marketplace(),
                    audience.entitled_among_any(&consumers),
                )
            },
            |m| {
                (
                    audience.users_reaching_marketplace(m),
                    audience.entitled_among(m, &consumers),
                )
            },
        );
        Self {
            spark_sum: (0..super::SPARK_DAYS)
                .map(|i| {
                    rows.iter()
                        .map(|r| spark_values(r, today)[usize::try_from(i).unwrap_or(0)])
                        .sum()
                })
                .collect(),
            completion,
            judged,
            installed: adoption.iter().map(|a| a.installed).sum(),
            installed_entitled: i64::try_from(installed_entitled).unwrap_or(0),
            entitled: i64::try_from(entitled).unwrap_or(0),
            active: rows.iter().map(|r| r.users).max().unwrap_or(0),
            invocations: rows.iter().map(|r| r.invocations).sum(),
            cost: rows.iter().map(|r| r.cost_microdollars).sum(),
        }
    }
}

fn tile(
    head: (&'static str, &'static str),
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
) -> SkillsKpiView {
    SkillsKpiView {
        label: head.0,
        icon: head.1,
        value,
        note,
        tone,
        hint,
        spark: sparkline_toned(&[], tone, String::new()),
    }
}

fn adoption_tiles(rows: &[SkillFactRow], g: &Figures) -> Vec<SkillsKpiView> {
    let sum = |f: fn(&SkillFactRow) -> i64| rows.iter().map(f).sum::<i64>();
    let outside = g.installed - g.installed_entitled;
    let mut tiles = vec![
        tile(
            ("Skills used", "skill"),
            rows.len().to_string(),
            format!("{} invocations in the window", g.invocations),
            "accent",
            "Distinct plugin:skill keys with at least one hook-reported invocation",
        ),
        tile(
            ("People", "people"),
            g.active.to_string(),
            format!("of {} entitled · {} installed", g.entitled, g.installed),
            "accent",
            "Most people any one skill reached; entitlement is resolved from the access-control rules",
        ),
        tile(
            ("Install rate", "download"),
            percent(g.installed_entitled, g.entitled),
            if outside > 0 {
                format!(
                    "{} of {} entitled hold a receipt · +{outside} outside",
                    g.installed_entitled, g.entitled
                )
            } else {
                format!(
                    "{} of {} entitled hold a receipt",
                    g.installed_entitled, g.entitled
                )
            },
            if g.entitled > 0 && g.installed_entitled * 2 >= g.entitled {
                "ok"
            } else {
                "warn"
            },
            "Entitled people holding a verified installation receipt, over everyone entitled; consumers the rules do not reach are counted beside it",
        ),
        tile(
            ("Conversations", "chat"),
            sum(|r| r.conversations).to_string(),
            format!(
                "{} turns · {} tools · {} artifacts",
                sum(|r| r.turns),
                sum(|r| r.tool_calls),
                sum(|r| r.artifacts)
            ),
            "accent",
            "Gateway conversations whose harness session invoked a skill",
        ),
    ];
    if let Some(head) = tiles.first_mut() {
        head.spark = sparkline_toned(&g.spark_sum, head.tone, "Invocations per day".to_owned());
    }
    tiles
}

fn spend_tiles(rows: &[SkillFactRow], g: &Figures) -> Vec<SkillsKpiView> {
    let sum = |f: fn(&SkillFactRow) -> i64| rows.iter().map(f).sum::<i64>();
    vec![
        tile(
            ("Cost", "coins"),
            format_cost(g.cost),
            format!(
                "{} per invocation",
                format_cost(g.cost / g.invocations.max(1))
            ),
            "accent",
            "Priced spend of the skill conversations",
        ),
        tile(
            ("Errors", "alert"),
            (sum(|r| r.errors)).to_string(),
            format!("failed requests · {} denied tool calls", sum(|r| r.denied)),
            error_rate_tone(sum(|r| r.errors), sum(|r| r.requests)),
            "Failed requests and governance denials inside skill conversations",
        ),
        tile(
            ("AI score", "sparkle"),
            score_display(g.completion),
            format!("{} conversations judged", g.judged),
            completion_tone(g.completion),
            "Mean of the judge's one completion score over judged skill conversations",
        ),
    ]
}

pub(super) fn kpis(
    rows: &[SkillFactRow],
    adoption: &[&MarketplaceAdoptionRow],
    audience: &MarketplaceAudience,
    marketplace: Option<&str>,
    today: NaiveDate,
) -> Vec<SkillsKpiView> {
    let figures = Figures::new(rows, adoption, audience, marketplace, today);
    let mut tiles = adoption_tiles(rows, &figures);
    tiles.extend(spend_tiles(rows, &figures));
    tiles
}

pub(super) fn unused(
    rows: &[SkillFactRow],
    index: &InventoryIndex,
    audience: &MarketplaceAudience,
    marketplace: Option<&str>,
) -> Vec<UnusedSkillView> {
    let used: std::collections::HashSet<&str> = rows.iter().map(|r| r.skill.as_str()).collect();
    let used = &used;
    let mut out: Vec<UnusedSkillView> = index
        .plugins_by_skill
        .iter()
        .flat_map(|(skill, plugins)| {
            plugins.iter().filter_map(move |p| {
                let key = format!("{}:{}", p.0, skill.replace('_', "-"));
                let placed = index
                    .marketplaces_by_plugin
                    .get(&p.0)
                    .and_then(|m| m.first())
                    .map(|m| m.id.clone())
                    .unwrap_or_default();
                let in_scope = marketplace.is_none_or(|m| m == placed);
                (in_scope && !used.contains(key.as_str())).then(|| UnusedSkillView {
                    catalog_href: format!("/admin/skills/{}", urlencoding::encode(skill)),
                    entitled: audience.users_reaching_skill(index, skill),
                    marketplace: placed,
                    plugin: p.0.clone(),
                    skill: key,
                })
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.entitled
            .cmp(&a.entitled)
            .then_with(|| a.skill.cmp(&b.skill))
    });
    out
}
