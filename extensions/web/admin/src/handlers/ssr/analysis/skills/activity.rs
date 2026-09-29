//! The Activity tab: invocations over the window as one chart, and the
//! skills it draws as a list.

use chrono::{Duration, NaiveDate};

use super::rows::marketplace_name;
use super::views::TopSkillView;
use super::{SkillsQuery, SkillsTab};
use crate::handlers::ssr::analysis::tone::percent;
use crate::handlers::ssr::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::skills::SkillFactRow;

const CHART_SERIES: usize = 6;

// Why: the row's `spark_days` hold every day in the window that saw an
// invocation; the chart wants every day, zeros included, oldest first, and
// weeks once a year is on screen so the line stays readable.
fn window_values(row: &SkillFactRow, today: NaiveDate, days: i64, step: i64) -> Vec<i64> {
    let by_day: std::collections::HashMap<NaiveDate, i64> = row
        .spark_days
        .iter()
        .copied()
        .zip(row.spark_counts.iter().copied())
        .collect();
    (0..days)
        .rev()
        .map(|back| {
            by_day
                .get(&(today - Duration::days(back)))
                .copied()
                .unwrap_or(0)
        })
        .collect::<Vec<i64>>()
        .chunks(usize::try_from(step).unwrap_or(1))
        .map(|chunk| chunk.iter().sum())
        .collect()
}

const fn chart_step(days: i64) -> i64 {
    if days > 90 { 7 } else { 1 }
}

pub(super) fn chart(rows: &[SkillFactRow], today: NaiveDate, days: i64) -> SvgLineChartView {
    let step = chart_step(days);
    let mut top: Vec<&SkillFactRow> = rows.iter().collect();
    top.sort_by_key(|r| std::cmp::Reverse(r.invocations));
    let labels: Vec<String> = (0..days)
        .rev()
        .step_by(usize::try_from(step).unwrap_or(1))
        .map(|back| (today - Duration::days(back)).format("%b %-d").to_string())
        .collect();
    let mut series: Vec<SvgSeriesInput> = top
        .iter()
        .take(CHART_SERIES)
        .map(|r| SvgSeriesInput {
            label: r.skill.clone(),
            values: window_values(r, today, days, step),
            value_display: r.invocations.to_string(),
        })
        .collect();
    if top.len() > CHART_SERIES {
        let mut rest = vec![0i64; labels.len()];
        for r in top.iter().skip(CHART_SERIES) {
            for (slot, n) in rest.iter_mut().zip(window_values(r, today, days, step)) {
                *slot += n;
            }
        }
        series.push(SvgSeriesInput {
            label: format!("{} other skills", top.len() - CHART_SERIES),
            value_display: rest.iter().sum::<i64>().to_string(),
            values: rest,
        });
    }
    let unit = if step == 1 { "day" } else { "week" };
    line_chart(LineChartSpec {
        title: "Invocations",
        subtitle: format!("Per {unit} over the last {days} days, top skills"),
        empty_message: "No skill was invoked in this window",
        series,
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: labels.first().cloned().unwrap_or_default(),
        x_mid_display: labels.get(labels.len() / 2).cloned().unwrap_or_default(),
        x_end_display: labels.last().cloned().unwrap_or_default(),
        show_area: true,
        x_labels: labels,
        y_unit: "",
    })
    .into_columns()
}

pub(super) fn top_skills(
    query: &SkillsQuery,
    rows: &[SkillFactRow],
    index: &InventoryIndex,
) -> Vec<TopSkillView> {
    let total: i64 = rows.iter().map(|r| r.invocations).sum::<i64>().max(1);
    let mut top: Vec<&SkillFactRow> = rows.iter().collect();
    top.sort_by_key(|r| std::cmp::Reverse(r.invocations));
    top.iter()
        .map(|r| TopSkillView {
            skill: r.skill.clone(),
            href: SkillsQuery {
                days: query.days,
                marketplace: query.marketplace.clone(),
                search: Some(r.skill.clone()),
                group: query.group.clone(),
                project: query.project.clone(),
                ..SkillsQuery::default()
            }
            .link(SkillsTab::Skills, None, None),
            marketplace_name: r.marketplace_id.as_ref().map_or_else(
                || "Unplaced".to_owned(),
                |m| marketplace_name(index, m.as_str()),
            ),
            invocations: r.invocations,
            users: r.users,
            share: percent(r.invocations, total),
            share_pct: (r.invocations * 100 / total).min(100),
            last_used: r.last_used.format("%b %-d, %H:%M").to_string(),
        })
        .collect()
}
