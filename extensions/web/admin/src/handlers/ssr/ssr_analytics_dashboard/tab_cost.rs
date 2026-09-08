//! View assembly for the Cost tab, in two audiences.
//!
//! The internal view is the operator's supplier bill: provider cost by day,
//! then by provider and by model. The customer view is what a container
//! consumed and carries no cost column at all, because the export it feeds is
//! sent outside the platform team — the repository selects no cost for it, so
//! this file has none to render even by accident.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::{StackSeriesInput, StackedChartSpec, stacked_chart};
use crate::repositories::analytics::site::cost::{
    ContainerAxis, ContainerUsageRow, CostDayRow, SupplierCostRow,
};
use crate::util::time_range::TimeRange;

use super::context::{ContainerRowView, CostTabView, KpiTile, SupplierRowView};
use super::tab_models::{per, share};
use super::view::{compact, date_label, midpoint};
use super::{AnalyticsDashboardQuery, urls};

pub(super) struct CostInput<'a> {
    pub days: &'a [CostDayRow],
    pub providers: &'a [SupplierCostRow],
    pub models: &'a [SupplierCostRow],
    pub containers: &'a [ContainerUsageRow],
    pub axis: ContainerAxis,
    pub is_internal: bool,
}

pub(super) fn cost_tab(
    input: &CostInput<'_>,
    range: &TimeRange,
    query: &AnalyticsDashboardQuery,
) -> CostTabView {
    let provider_max = input
        .providers
        .iter()
        .map(|p| p.cost_microdollars)
        .max()
        .unwrap_or(0);
    let model_max = input
        .models
        .iter()
        .map(|m| m.cost_microdollars)
        .max()
        .unwrap_or(0);
    let container_max = input
        .containers
        .iter()
        .map(|c| c.requests)
        .max()
        .unwrap_or(0);

    CostTabView {
        kpis: kpis(input),
        provider_count: input.providers.len(),
        model_count: input.models.len(),
        container_count: input.containers.len(),
        audience_links: urls::audience_links(query, input.is_internal),
        is_internal: input.is_internal,
        csv_url: urls::cost_csv_url(query, input.is_internal),
        day_chart: input.is_internal.then(|| day_chart(input.days, range)),
        has_providers: !input.providers.is_empty(),
        providers: input
            .providers
            .iter()
            .map(|p| {
                supplier_row(
                    p,
                    provider_max,
                    urls::drill_url(query, "provider", &p.label),
                )
            })
            .collect(),
        has_models: !input.models.is_empty(),
        models: input
            .models
            .iter()
            .map(|m| supplier_row(m, model_max, urls::drill_url(query, "model", &m.label)))
            .collect(),
        axis_links: urls::axis_links(query, input.axis),
        has_containers: !input.containers.is_empty(),
        containers: input
            .containers
            .iter()
            .map(|c| container_row(c, container_max, input.axis))
            .collect(),
        axis_label: match input.axis {
            ContainerAxis::Group => "Group",
            ContainerAxis::Project => "Project",
        },
    }
}

fn supplier_row(row: &SupplierCostRow, max: i64, drill_url: String) -> SupplierRowView {
    SupplierRowView {
        label: row.label.clone(),
        requests: row.requests,
        share_pct: share(row.cost_microdollars, max),
        tokens_display: compact(row.tokens),
        cost_display: format_cost(row.cost_microdollars),
        drill_url,
    }
}

fn container_row(row: &ContainerUsageRow, max: i64, axis: ContainerAxis) -> ContainerRowView {
    // Why: the bucket for rollup rows whose owner had no primary container —
    // it is a row on the table, never a silent omission from the totals.
    let is_unattributed = row.container_id == "unattributed";
    let param = match axis {
        ContainerAxis::Group => "group",
        ContainerAxis::Project => "project",
    };
    ContainerRowView {
        users: row.users,
        sessions: row.sessions,
        requests: row.requests,
        share_pct: share(row.requests, max),
        input_display: compact(row.input_tokens),
        output_display: compact(row.output_tokens),
        is_unattributed,
        drill_url: if is_unattributed {
            "/admin/analytics?tab=cost".to_owned()
        } else {
            format!(
                "/admin/analytics?tab=cost&{param}={}",
                urlencoding::encode(&row.container_id)
            )
        },
        container_id: row.container_id.clone(),
    }
}

fn day_chart(
    days: &[CostDayRow],
    range: &TimeRange,
) -> crate::handlers::ssr::types::SvgStackedChartView {
    let mut labels: Vec<chrono::NaiveDate> = days.iter().map(|d| d.day).collect();
    labels.sort_unstable();
    labels.dedup();
    let mut providers: Vec<String> = days.iter().map(|d| d.provider.clone()).collect();
    providers.sort();
    providers.dedup();

    let mut series: Vec<StackSeriesInput> = providers
        .iter()
        .map(|p| {
            let values: Vec<i64> = labels
                .iter()
                .map(|day| {
                    days.iter()
                        .filter(|d| d.day == *day && d.provider == *p)
                        .map(|d| d.cost_microdollars)
                        .sum()
                })
                .collect();
            let total: i64 = values.iter().sum();
            StackSeriesInput {
                label: p.clone(),
                values,
                value_display: format_cost(total),
            }
        })
        .collect();
    series.sort_by_key(|s| std::cmp::Reverse(s.values.iter().sum::<i64>()));

    let grand: i64 = days.iter().map(|d| d.cost_microdollars).sum();
    stacked_chart(StackedChartSpec {
        title: "Provider cost by day",
        subtitle: format!(
            "{} across {} providers",
            format_cost(grand),
            providers.len()
        ),
        empty_message: "No billed requests in this window.",
        series,
        bucket_labels: labels
            .iter()
            .map(|d| d.format("%b %d").to_string())
            .collect(),
        value_display: format_cost,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
    })
}

fn kpis(input: &CostInput<'_>) -> Vec<KpiTile> {
    let cost: i64 = input.providers.iter().map(|p| p.cost_microdollars).sum();
    let requests: i64 = input.providers.iter().map(|p| p.requests).sum();
    let tokens: i64 = input.providers.iter().map(|p| p.tokens).sum();
    let containers = input.containers.len();
    let unattributed: i64 = input
        .containers
        .iter()
        .filter(|c| c.container_id == "unattributed")
        .map(|c| c.requests)
        .sum();

    if input.is_internal {
        return vec![
            KpiTile {
                label: "Provider cost".to_owned(),
                value: format_cost(cost),
                sub: format!("{requests} billable requests"),
                tone: "accent",
            },
            KpiTile {
                label: "Cost per request".to_owned(),
                value: format_cost(per(cost, requests)),
                sub: "supplier price, this window".to_owned(),
                tone: "ok",
            },
            KpiTile {
                label: "Tokens".to_owned(),
                value: compact(tokens),
                sub: "input plus output".to_owned(),
                tone: "ok",
            },
            KpiTile {
                label: "Providers".to_owned(),
                value: input.providers.len().to_string(),
                sub: format!("{} models served", input.models.len()),
                tone: "accent",
            },
        ];
    }
    vec![
        KpiTile {
            label: "Containers".to_owned(),
            value: containers.to_string(),
            sub: "with consumption in this window".to_owned(),
            tone: "accent",
        },
        KpiTile {
            label: "Requests".to_owned(),
            value: compact(input.containers.iter().map(|c| c.requests).sum()),
            sub: "from the daily rollups".to_owned(),
            tone: "ok",
        },
        KpiTile {
            label: "Unattributed".to_owned(),
            value: compact(unattributed),
            sub: "requests by people with no primary container".to_owned(),
            tone: if unattributed > 0 { "warn" } else { "ok" },
        },
        KpiTile {
            label: "Cost shown".to_owned(),
            value: "none".to_owned(),
            sub: "the customer view carries no supplier figure".to_owned(),
            tone: "ok",
        },
    ]
}
