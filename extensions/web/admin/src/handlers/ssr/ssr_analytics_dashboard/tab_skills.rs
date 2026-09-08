//! View assembly for the Skills tab.
//!
//! The cost column is measured or blank. A skill invoked as a slash command
//! produces no tool call, so no gateway request can be tied to it, and the
//! row shows a dash beside an explicit count of what is unattributed. The KPI
//! strip leads with that share so nobody reads the cost column as the total.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::site::skills::{SkillModelRow, SkillStatsRow, SkillTotals};

use super::context::{KpiTile, SkillModelRowView, SkillRowView, SkillsTabView};
use super::tab_models::{pct, share, short_name};
use super::view::compact;
use super::{AnalyticsDashboardQuery, PAGE_SIZE, urls};

pub(super) struct SkillsInput<'a> {
    pub rows: &'a [SkillStatsRow],
    pub total_rows: i64,
    pub by_model: &'a [SkillModelRow],
    pub totals: SkillTotals,
    pub page: i64,
}

pub(super) fn skills_tab(
    input: &SkillsInput<'_>,
    query: &AnalyticsDashboardQuery,
) -> SkillsTabView {
    let max = input.rows.iter().map(|r| r.invocations).max().unwrap_or(0);
    let views: Vec<SkillRowView> = input
        .rows
        .iter()
        .map(|r| SkillRowView {
            skill: r.skill.clone(),
            name_display: short_name(&r.skill, ':'),
            invocations: r.invocations,
            share_pct: share(r.invocations, max),
            slash_display: r.slash_invocations.to_string(),
            tool_display: r.tool_invocations.to_string(),
            users: r.distinct_users,
            sessions: r.distinct_sessions,
            // Why: blank, not zero. Zero is a measurement; a dash is the
            // absence of one, and these two are not the same claim.
            cost_display: if r.measured_invocations > 0 {
                format_cost(r.measured_cost_microdollars)
            } else {
                "—".to_owned()
            },
            unattributed_display: unattributed(r),
            rating_display: r
                .avg_rating
                .map_or_else(|| "—".to_owned(), |v| format!("{v:.1} ({})", r.ratings)),
            // Why: NOT the request log. No skill invocation on this instance
            // can be joined to a gateway request — the join needs a `Skill`
            // tool call and every invocation here is a slash command — so a
            // `skill=` filter on that log would answer with everything or with
            // nothing, and both read as a fact. The skill's own page is the
            // honest destination until the measured link exists.
            drill_url: format!(
                "/admin/skills/{}",
                urlencoding::encode(&short_name(&r.skill, ':'))
            ),
        })
        .collect();

    let pagination = (input.total_rows > PAGE_SIZE).then(|| {
        urls::build_pagination(
            query,
            PageWindow::new(
                input.page,
                PAGE_SIZE,
                input.total_rows,
                i64::try_from(input.rows.len()).unwrap_or(PAGE_SIZE),
                "skills",
            ),
        )
    });

    let by_model: Vec<SkillModelRowView> = input
        .by_model
        .iter()
        .map(|r| SkillModelRowView {
            skill: r.skill.clone(),
            model: r.model.clone(),
            requests: r.requests,
            cost_display: format_cost(r.cost_microdollars),
            drill_url: urls::drill_url(query, "model", &r.model),
        })
        .collect();

    SkillsTabView {
        kpis: kpis(input.totals),
        skill_count: input.total_rows,
        by_model_count: by_model.len(),
        has_rows: !views.is_empty(),
        rows: views,
        pagination,
        has_by_model: !by_model.is_empty(),
        by_model,
        measurement_note: measurement_note(input.totals),
    }
}

fn unattributed(r: &SkillStatsRow) -> String {
    let n = r.unattributed_invocations();
    if n == 0 {
        "—".to_owned()
    } else {
        n.to_string()
    }
}

fn kpis(totals: SkillTotals) -> Vec<KpiTile> {
    let unmeasured = totals.invocations - totals.measured_invocations;
    let share = pct(totals.measured_invocations, totals.invocations);
    vec![
        KpiTile {
            label: "Invocations".to_owned(),
            value: compact(totals.invocations),
            sub: format!("{} distinct skills", totals.distinct_skills),
            tone: "accent",
        },
        KpiTile {
            label: "People using skills".to_owned(),
            value: totals.distinct_users.to_string(),
            sub: "distinct users in window".to_owned(),
            tone: "ok",
        },
        KpiTile {
            label: "Cost measured".to_owned(),
            value: format!("{share:.0}%"),
            sub: format!(
                "{} of {} invocations",
                totals.measured_invocations, totals.invocations
            ),
            tone: if share >= 50.0 { "ok" } else { "warn" },
        },
        KpiTile {
            label: "Unattributed".to_owned(),
            value: compact(unmeasured),
            sub: "slash invocations carry no tool call to price".to_owned(),
            tone: if unmeasured > 0 { "warn" } else { "ok" },
        },
    ]
}

fn measurement_note(totals: SkillTotals) -> String {
    let unmeasured = totals.invocations - totals.measured_invocations;
    if unmeasured == 0 {
        "Every invocation in this window carries a measured gateway cost.".to_owned()
    } else {
        format!(
            "{unmeasured} of {} invocations carry no gateway request that can be priced, so \
             their cost cells are blank rather than estimated. A skill typed as a slash \
             command sends no tool call, which is the usual reason.",
            totals.invocations
        )
    }
}
