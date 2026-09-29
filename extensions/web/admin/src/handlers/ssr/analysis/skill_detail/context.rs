//! Assembling the skill detail page context.

use super::figures::{charts, kpis};
use super::views::{BucketView, SkillDetailContext};
use super::{PAGE_SIZE, SkillDetailQuery, SkillRef};
use crate::handlers::ssr::analysis::conversations::view::ConversationRowView;
use crate::handlers::ssr::analysis::skills::SkillFactView;
use crate::handlers::ssr::analysis::tone::{
    completion_tone, error_rate_tone, latency_tone, score_display,
};
use crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::handlers::ssr::list_view::{PageWindow, Pagination, paginate};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::skills::{
    SkillBreakdownBy, SkillBucketRow, SkillConversationRow, SkillDayRow, SkillFactRow, SkillRunRow,
};

// Why: the hook carries ids; the inventory index resolves the plugin's
// display name and the marketplaces that include it.
pub(super) fn resolve_names(
    index: &InventoryIndex,
    skill: &SkillRef,
) -> (String, String, Vec<String>) {
    let skill_name = skill
        .skill
        .split_once(':')
        .map_or_else(|| skill.skill.clone(), |(_, s)| s.to_owned());
    let plugin_name = index
        .plugins_by_skill
        .get(&skill_name.replace('-', "_"))
        .and_then(|list| list.iter().find(|p| p.0 == skill.plugin_id.as_str()))
        .map_or_else(|| skill.plugin_id.as_str().to_owned(), |p| p.1.clone());
    let marketplaces = index
        .marketplaces_by_plugin
        .get(skill.plugin_id.as_str())
        .map(|list| list.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();
    (skill_name, plugin_name, marketplaces)
}

// Why: what the repositories read for the page, handed to the builder as one.
pub(super) struct SkillRead<'a> {
    pub(super) row: Option<SkillFactView>,
    pub(super) facts: Option<&'a SkillFactRow>,
    pub(super) daily: &'a [SkillDayRow],
    pub(super) breakdown: &'a [SkillBucketRow],
    pub(super) conversations: &'a [SkillConversationRow],
    pub(super) runs: &'a [SkillRunRow],
    pub(super) total: i64,
    // Why: whether the viewer may press a row's Judge button.
    pub(super) can_judge: bool,
}

fn range_links(query: &SkillDetailQuery, key: &str) -> Vec<TabLinkView> {
    [7, 30, 90, 365]
        .iter()
        .map(|d| TabLinkView {
            slug: match d {
                7 => "7",
                90 => "90",
                365 => "365",
                _ => "30",
            },
            label: match d {
                7 => "7 days",
                90 => "90 days",
                365 => "1 year",
                _ => "30 days",
            },
            href: query.link(key, Some(*d), None, None),
            is_active: *d == query.days(),
            count: None,
        })
        .collect()
}

fn breakdown_tabs(query: &SkillDetailQuery, key: &str) -> Vec<TabLinkView> {
    SkillBreakdownBy::ALL
        .iter()
        .map(|by| TabLinkView {
            slug: by.as_str(),
            label: by.label(),
            href: query.link(key, None, Some(*by), None),
            is_active: *by == query.by(),
            count: None,
        })
        .collect()
}

fn pagination(query: &SkillDetailQuery, key: &str, total: i64, shown: usize) -> Pagination {
    let shown = i64::try_from(shown).unwrap_or(0);
    let window = PageWindow::new(query.page(), PAGE_SIZE, total, shown, "conversations");
    paginate(window, |page| query.link(key, None, None, Some(page)))
}

pub(super) fn build_context(
    query: &SkillDetailQuery,
    skill: &SkillRef,
    index: &InventoryIndex,
    read: SkillRead<'_>,
) -> SkillDetailContext {
    let (skill_name, plugin_name, marketplaces) = resolve_names(index, skill);
    let entitled = read.row.as_ref().map_or(0, |r| r.entitled);
    let invocations_total: i64 = read.breakdown.iter().map(|b| b.invocations).sum();
    let key = skill.skill.as_str();
    let back_url = query.link(key, None, None, None);
    SkillDetailContext {
        page: "analysis-skill",
        title: format!("{skill_name} · Skill"),
        breadcrumbs: vec![
            BreadcrumbView::link("Analysis", ANALYSIS_SKILLS_URL),
            BreadcrumbView::link("Skills", ANALYSIS_SKILLS_URL),
            BreadcrumbView::current(skill_name.clone()),
        ],
        skill_key: skill.skill.clone(),
        skill_name: skill_name.clone(),
        plugin: skill.plugin_id.as_str().to_owned(),
        plugin_name,
        marketplaces_display: if marketplaces.is_empty() {
            "no marketplace".to_owned()
        } else {
            marketplaces.join(", ")
        },
        catalog_href: format!(
            "/admin/skills/{}",
            urlencoding::encode(&skill_name.replace('-', "_"))
        ),
        skills_href: ANALYSIS_SKILLS_URL,
        days: query.days(),
        range_links: range_links(query, key),
        used: read.facts.is_some(),
        kpis: kpis(read.facts, entitled),
        row: read.row,
        charts: charts(read.daily),
        breakdown_tabs: breakdown_tabs(query, key),
        breakdown_label: query.by().label(),
        breakdown: read
            .breakdown
            .iter()
            .map(|b| bucket_view(b, invocations_total))
            .collect(),
        releases: super::runs::release_views(read.runs, chrono::Utc::now()),
        runs: super::runs::run_views(read.runs, chrono::Utc::now()),
        run_count: read.runs.len(),
        conversations: read
            .conversations
            .iter()
            .map(|r| ConversationRowView::from_skill_row(r).with_viewer(read.can_judge, &back_url))
            .collect(),
        conversation_count: read.total,
        pagination: pagination(query, key, read.total, read.conversations.len()),
        export: crate::export::ExportView::new(
            &[
                "analysis-skill-conversations",
                "analysis-skill-runs",
                "analysis-kit-release-impact",
            ],
            &format!("skill={}&days={}", urlencoding::encode(key), query.days()),
        ),
        help: crate::handlers::ssr::analysis::help::skill_detail_help(),
    }
}

fn bucket_view(b: &SkillBucketRow, total: i64) -> BucketView {
    BucketView {
        label: b.label.clone(),
        user_href: b
            .user_id
            .as_ref()
            .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
        invocations: b.invocations,
        share_pct: if total > 0 {
            (b.invocations * 100 / total).min(100)
        } else {
            0
        },
        users: b.users,
        conversations: b.conversations,
        tokens_display: format_token_total(b.tokens),
        cost_display: format_cost(b.cost_microdollars),
        errors: b.errors,
        errors_tone: error_rate_tone(b.errors, b.conversations.max(1)),
        latency_display: b.p95_latency_ms.map_or_else(
            || "—".to_owned(),
            |ms| format_duration_ms(ms.round() as i64),
        ),
        latency_tone: latency_tone(b.p95_latency_ms),
        judged: b.judged,
        completion_display: score_display(b.completion_avg),
        completion_tone: completion_tone(b.completion_avg),
    }
}
