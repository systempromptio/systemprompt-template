//! Row shaping for the project detail page's Usage and Settings tabs.
//!
//! Sibling of [`super::detail`], which owns the reads and the page context.
//! Everything here turns one repository row into one rendered row, so the
//! templates do no arithmetic and no formatting decision lives in two places.

use crate::repositories::people_usage::DailyRequests;
use crate::repositories::projects::activity::{
    ProjectCommitRow, ProjectSessionRow, SkillEffectivenessRow, ToolHealthRow,
};
use crate::types::access_control::{AccessControlRule, AccessDecision};

use super::super::format::short_num;
use super::super::people_view::{format_usd, model_rows};
use super::super::types::{
    AccessRowView, LineChartSpec, ProjectCommitRowView, ProjectKpiView, ProjectSessionRowView,
    ProjectSkillRowView, ProjectToolRowView, ProjectUsageTabView, SvgLineChartView, SvgSeriesInput,
    TabLinkView, line_chart,
};
use super::detail::{DetailData, ProjectUsageData};
use super::pct;

pub(super) fn tabs(project_id: &str, active: &str) -> Vec<TabLinkView> {
    [
        ("members", "Members"),
        ("usage", "Usage"),
        ("settings", "Settings"),
    ]
    .into_iter()
    .map(|(slug, label)| TabLinkView {
        slug,
        label,
        href: format!("/admin/projects/{project_id}?tab={slug}"),
        is_active: slug == active,
        count: None,
    })
    .collect()
}

pub(super) fn kpis(data: &DetailData, member_count: i64) -> Vec<ProjectKpiView> {
    let usage = &data.usage;
    let success = pct(data.tool_success, data.tool_calls);
    let per_member = if member_count > 0 {
        usage.cost_microdollars / member_count
    } else {
        0
    };
    vec![
        tile(
            "Members",
            member_count.to_string(),
            format!("{} active in the window", usage.active_members),
            "accent",
        ),
        tile(
            "Requests",
            short_num(usage.requests),
            "attributed to this project alone",
            "accent",
        ),
        tile(
            "Tokens",
            format!(
                "{} / {}",
                short_num(usage.tokens_in),
                short_num(usage.tokens_out)
            ),
            "input / output over the window",
            "accent",
        ),
        tile(
            "Cost",
            format_usd(usage.cost_microdollars),
            format!("{} per member", format_usd(per_member)),
            "accent",
        ),
        tile(
            "Tool success",
            format!("{success}%"),
            format!("{} of {} MCP calls", data.tool_success, data.tool_calls),
            if data.tool_calls == 0 {
                "accent"
            } else if success >= 95 {
                "ok"
            } else if success >= 80 {
                "warn"
            } else {
                "err"
            },
        ),
        tile(
            "Skills",
            data.skills.len().to_string(),
            "distinct skills invoked",
            "accent",
        ),
    ]
}

fn tile(label: &str, value: String, note: impl Into<String>, tone: &'static str) -> ProjectKpiView {
    ProjectKpiView {
        label: label.to_owned(),
        value,
        note: note.into(),
        tone,
        href: None,
    }
}

pub(super) fn usage_tab(
    u: &ProjectUsageData,
    skills: &[SkillEffectivenessRow],
) -> ProjectUsageTabView {
    ProjectUsageTabView {
        daily: daily_chart(&u.daily),
        daily_cost: daily_cost_chart(&u.daily),
        model_count: u.models.len() as i64,
        skill_count: skills.len() as i64,
        tool_count: u.tools.len() as i64,
        session_count: u.sessions.len() as i64,
        commit_count: u.commits.len() as i64,
        models: model_rows(&u.models),
        skills: skill_rows(skills),
        tools: tool_rows(&u.tools),
        sessions: u.sessions.iter().map(session_row).collect(),
        commits: u.commits.iter().map(commit_row).collect(),
        commit_files: u
            .commits
            .iter()
            .map(|c| i64::from(c.files_changed.unwrap_or(0)))
            .sum(),
        commit_insertions: u
            .commits
            .iter()
            .map(|c| i64::from(c.insertions.unwrap_or(0)))
            .sum(),
        commit_deletions: u
            .commits
            .iter()
            .map(|c| i64::from(c.deletions.unwrap_or(0)))
            .sum(),
    }
}

fn daily_chart(daily: &[DailyRequests]) -> SvgLineChartView {
    let requests: Vec<i64> = daily.iter().map(|d| d.requests).collect();
    let total: i64 = requests.iter().sum();
    let cost: i64 = daily.iter().map(|d| d.cost_microdollars).sum();
    let day = |i: usize| daily.get(i).map_or_else(String::new, |d| d.day.to_string());
    line_chart(LineChartSpec {
        title: "Requests per day",
        subtitle: format!("{total} requests, {} over the window", format_usd(cost)),
        empty_message: "No gateway traffic attributed to this project in the window.",
        series: vec![SvgSeriesInput {
            label: "Requests".to_owned(),
            values: requests,
            value_display: total.to_string(),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: day(0),
        x_mid_display: day(daily.len() / 2),
        x_end_display: day(daily.len().saturating_sub(1)),
        show_area: true,
    })
}

// Why: cost gets its own plot rather than a second series on the request
// chart — two units on one axis would make the cheaper line look like less
// work rather than less money.
fn daily_cost_chart(daily: &[DailyRequests]) -> SvgLineChartView {
    let cost: Vec<i64> = daily.iter().map(|d| d.cost_microdollars).collect();
    let total: i64 = cost.iter().sum();
    let day = |i: usize| daily.get(i).map_or_else(String::new, |d| d.day.to_string());
    line_chart(LineChartSpec {
        title: "Cost per day",
        subtitle: format!("{} over the window", format_usd(total)),
        empty_message: "Nothing was billed to this project in the window.",
        series: vec![SvgSeriesInput {
            label: "Cost".to_owned(),
            values: cost,
            value_display: format_usd(total),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: format_usd,
        x_start_display: day(0),
        x_mid_display: day(daily.len() / 2),
        x_end_display: day(daily.len().saturating_sub(1)),
        show_area: true,
    })
}

pub(super) fn skill_rows(rows: &[SkillEffectivenessRow]) -> Vec<ProjectSkillRowView> {
    rows.iter()
        .map(|r| ProjectSkillRowView {
            skill: r.skill.clone(),
            invocations: r.invocations,
            users: r.users,
            rating_display: r
                .rating_avg
                .map_or_else(|| "—".to_owned(), |avg| format!("{avg:.1} / 5")),
            rating_count: r.rating_count,
        })
        .collect()
}

pub(super) fn tool_rows(rows: &[ToolHealthRow]) -> Vec<ProjectToolRowView> {
    rows.iter()
        .map(|r| {
            let error_pct = pct(r.failures, r.calls);
            ProjectToolRowView {
                server_name: r.server_name.clone(),
                tool_name: r.tool_name.clone(),
                calls: r.calls,
                failures: r.failures,
                users: r.users,
                error_pct,
                tone: if error_pct == 0 {
                    "ok"
                } else if error_pct < 10 {
                    "warn"
                } else {
                    "err"
                },
                p95_display: format!("{} ms", r.p95_ms),
            }
        })
        .collect()
}

pub(super) fn session_row(s: &ProjectSessionRow) -> ProjectSessionRowView {
    ProjectSessionRowView {
        session_short: short(s.session_id.as_str()),
        href: format!("/admin/sessions/{}", s.session_id.as_str()),
        session_id: s.session_id.clone(),
        user_id: s.user_id.clone(),
        requests: s.requests,
        models: s.models,
        cost_display: format_usd(s.cost_microdollars),
        last_activity: s.last_activity_at.to_rfc3339(),
    }
}

pub(super) fn commit_row(c: &ProjectCommitRow) -> ProjectCommitRowView {
    ProjectCommitRowView {
        commit_short: short(&c.commit_hash),
        user_id: c.user_id.clone(),
        branch: c.branch.clone().unwrap_or_else(|| "—".to_owned()),
        message: c.message.clone(),
        files_changed: c.files_changed.unwrap_or(0),
        insertions: c.insertions.unwrap_or(0),
        deletions: c.deletions.unwrap_or(0),
        committed_at: c.committed_at.to_rfc3339(),
    }
}

// Why: identifiers are read for recognition, not transcription — the full
// value stays on the row as its title attribute.
fn short(id: &str) -> String {
    id.chars().take(8).collect()
}

pub(super) fn gated_rows(rules: &[AccessControlRule]) -> Vec<AccessRowView> {
    rules
        .iter()
        .map(|r| AccessRowView {
            entity_type: r.entity_type.clone(),
            entity_id: r.entity_id.clone(),
            entity_name: r.entity_id.clone(),
            effective: match r.access {
                AccessDecision::Allow => "allow".to_owned(),
                AccessDecision::Deny => "deny".to_owned(),
            },
            layer: "project".to_owned(),
            detail: format!("project:{} {}", r.rule_value, r.entity_type),
            state: match r.access {
                AccessDecision::Allow => "allow",
                AccessDecision::Deny => "deny",
            },
        })
        .collect()
}
