//! View assembly for the Skills tab.
//!
//! A skill has no cost of its own — it is instructions pasted into a
//! conversation — so the tab shows adoption (invocations, people,
//! conversations) and then facts about the conversations that used the skill:
//! their requests and spend, labelled as conversation spend and stated to
//! overlap. Each row links to the Analysis section for the conversation-level
//! detail so both screens tell one story.

use crate::handlers::ssr::analysis_urls::analysis_skill_url;
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::site::skills::{SkillStatsRow, SkillTotals};

use super::context::{KpiTile, SkillRowView, SkillsTabView};
use super::tab_models::{pct, share, short_name};
use super::view::compact;
use super::{AnalyticsDashboardQuery, PAGE_SIZE, urls};

pub(super) struct SkillsInput<'a> {
    pub rows: &'a [SkillStatsRow],
    pub total_rows: i64,
    pub totals: SkillTotals,
    pub page: i64,
}

pub(super) fn skills_tab(
    input: &SkillsInput<'_>,
    query: &AnalyticsDashboardQuery,
) -> SkillsTabView {
    let max = input.rows.iter().map(|r| r.invocations).max().unwrap_or(0);
    let views: Vec<SkillRowView> = input.rows.iter().map(|r| row_view(r, max)).collect();

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

    SkillsTabView {
        kpis: kpis(input.totals),
        skill_count: input.total_rows,
        has_rows: !views.is_empty(),
        rows: views,
        pagination,
        measurement_note: measurement_note(input.totals),
    }
}

fn row_view(r: &SkillStatsRow, max: i64) -> SkillRowView {
    let catalog_url = format!(
        "/admin/skills/{}",
        urlencoding::encode(&short_name(&r.skill, ':').replace('-', "_"))
    );
    SkillRowView {
        skill: r.skill.clone(),
        name_display: short_name(&r.skill, ':'),
        invocations: r.invocations,
        share_pct: share(r.invocations, max),
        slash_display: r.slash_invocations.to_string(),
        tool_display: r.tool_invocations.to_string(),
        users: r.distinct_users,
        conversations: r.conversations,
        requests: r.requests,
        // Why: blank, not zero. A conversation with no priced request has an
        // unknown spend, and zero would be a measurement it does not have.
        cost_display: if r.priced_requests > 0 {
            format_cost(r.conversation_cost_microdollars)
        } else {
            "—".to_owned()
        },
        attributed_display: format!("{:.0}%", pct(r.attributed_invocations, r.invocations)),
        // Why: the skill's Analysis page lists the conversations behind this
        // row and the judge's scores; a resolved managed resource narrows it
        // to revision-verified evidence.
        analysis_url: Some(format!(
            "{}{}",
            analysis_skill_url(&r.skill),
            r.resource_id
                .as_deref()
                .map(|id| format!("?resource={}", urlencoding::encode(id)))
                .unwrap_or_default()
        )),
        catalog_url,
    }
}

fn kpis(totals: SkillTotals) -> Vec<KpiTile> {
    let attributed = pct(totals.attributed_invocations, totals.invocations);
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
            label: "Conversations".to_owned(),
            value: compact(totals.conversations),
            sub: format!("{} gateway requests", compact(totals.requests)),
            tone: "accent",
        },
        KpiTile {
            label: "Conversation spend".to_owned(),
            value: format_cost(totals.conversation_cost_microdollars),
            sub: "each conversation counted once".to_owned(),
            tone: "accent",
        },
        KpiTile {
            label: "Attributed to a version".to_owned(),
            value: format!("{attributed:.0}%"),
            sub: format!(
                "{} of {} invocations via a verified install",
                totals.attributed_invocations, totals.invocations
            ),
            tone: if attributed >= 50.0 { "ok" } else { "warn" },
        },
    ]
}

fn measurement_note(totals: SkillTotals) -> String {
    format!(
        "A skill is instructions pasted into a conversation and has no cost of its own. \
         Each row shows the requests and spend of the {} conversations that invoked it, \
         from the same facts as Analysis. A conversation that used several skills is \
         counted under each, so rows overlap and must not be summed; the strip above \
         counts every conversation and request once.",
        totals.conversations
    )
}
