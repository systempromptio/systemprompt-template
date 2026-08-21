//! View-model assembly for the per-user analytics page. Pure functions from
//! repository rows to the typed context; all label and percentage math lives
//! here so the template derives nothing.

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::{
    LineChartSpec, PieSliceInput, PieView, SvgLineChartView, SvgSeriesInput, delta_view,
    line_chart, pie_view,
};
use crate::types::UserDetail;
use crate::util::time_range::TimeRange;

use super::context::{
    AnalyticsUserContext, UserCodeTotalsView, UserDailyRowView, UserKpiView, UserSessionRowView,
    UserTimeRange,
};
use super::data::AnalyticsUserData;

pub(super) struct PageInput<'a> {
    pub user_id: &'a UserId,
    pub detail: &'a UserDetail,
    pub range: TimeRange,
    pub query: &'a super::AnalyticsUserQuery,
    pub fetched: &'a AnalyticsUserData,
}

pub(super) fn page_context(input: &PageInput<'_>) -> AnalyticsUserContext {
    let &PageInput {
        user_id,
        detail,
        range,
        query,
        fetched,
    } = input;

    let label = detail
        .display_name
        .clone()
        .unwrap_or_else(|| user_id.as_str().to_owned());
    let encoded = urlencoding::encode(user_id.as_str()).into_owned();
    let base_url = format!("/admin/analytics/users/{encoded}");

    let title = format!("Analytics · {label}");
    AnalyticsUserContext {
        page: "analytics-user",
        title,
        user_id: user_id.clone(),
        label,
        email: detail
            .email
            .as_ref()
            .map_or_else(String::new, |e| e.as_str().to_owned()),
        roles_display: detail.roles.join(", "),
        time_range: UserTimeRange {
            preset: query.preset.clone().unwrap_or_else(|| "30d".to_owned()),
            from: range.from.to_rfc3339(),
            to: range.to.to_rfc3339(),
            base_url,
            query: String::new(),
        },

        kpis: kpi_view(fetched, &range),
        trend_chart: trend_chart(fetched, &range),
        model_pie: model_pie(fetched),
        code_chart: code_chart(fetched, &range),

        code_totals: code_totals(fetched),
        has_daily_rows: !fetched.daily.is_empty(),
        daily_rows: daily_rows(fetched),
        has_session_rows: !fetched.sessions.is_empty(),
        session_rows: session_rows(fetched),

        log_url: format!("/admin/entities/requests?tab=log&user_id={encoded}"),
        manage_url: format!("/admin/access/user?id={encoded}"),
        dashboard_url: format!("/admin/analytics?tab=usage&user_id={encoded}"),
    }
}

fn kpi_view(fetched: &AnalyticsUserData, range: &TimeRange) -> UserKpiView {
    let k = &fetched.kpis;
    let days = ((range.to - range.from).num_seconds().max(1) as f64 / 86_400.0).max(1.0);
    let error_rate = if k.total_requests > 0 {
        k.error_count as f64 / k.total_requests as f64 * 100.0
    } else {
        0.0
    };
    let grant_rate = if fetched.permissions.requests > 0 {
        fetched.permissions.granted as f64 / fetched.permissions.requests as f64 * 100.0
    } else {
        0.0
    };
    UserKpiView {
        requests: k.total_requests,
        requests_delta: delta_view(k.total_requests, k.prev_total_requests, true),
        cost_display: format_cost(k.total_cost_microdollars),
        cost_delta: delta_view(
            k.total_cost_microdollars,
            k.prev_total_cost_microdollars,
            false,
        ),
        tokens_display: compact(k.total_tokens),
        tokens_delta: delta_view(k.total_tokens, k.prev_total_tokens, true),
        error_display: format!("{} errors ({error_rate:.1}%)", k.error_count),
        requests_per_day_display: format!("{:.1}", k.total_requests as f64 / days),
        grant_rate_display: format!("{grant_rate:.0}%"),
        grant_has_data: fetched.permissions.requests > 0,
    }
}

fn trend_chart(fetched: &AnalyticsUserData, range: &TimeRange) -> SvgLineChartView {
    let requests: i64 = fetched.series.iter().map(|b| b.requests).sum();
    let cost: i64 = fetched.series.iter().map(|b| b.cost_microdollars).sum();
    line_chart(LineChartSpec {
        title: "Requests per day",
        subtitle: format!("{requests} requests · {} spent", format_cost(cost)),
        empty_message: "No gateway requests from this user in this window.",
        series: vec![SvgSeriesInput {
            label: "requests/day".to_owned(),
            values: fetched.series.iter().map(|b| b.requests).collect(),
            value_display: requests.to_string(),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: date_label(range.from),
        x_mid_display: date_label(range.from + (range.to - range.from) / 2),
        x_end_display: date_label(range.to),
        show_area: true,
    })
}

fn code_chart(fetched: &AnalyticsUserData, range: &TimeRange) -> SvgLineChartView {
    let ai: i64 = fetched.code_series.iter().map(|b| b.loc_added_ai).sum();
    let committed: i64 = fetched
        .code_series
        .iter()
        .map(|b| b.commit_insertions)
        .sum();
    line_chart(LineChartSpec {
        title: "AI lines vs committed lines",
        subtitle: format!(
            "{} AI lines applied · {} lines committed (different measurement frames)",
            compact(ai),
            compact(committed)
        ),
        empty_message: "No code activity recorded for this user in this window.",
        series: vec![
            SvgSeriesInput {
                label: "AI lines added".to_owned(),
                values: fetched.code_series.iter().map(|b| b.loc_added_ai).collect(),
                value_display: compact(ai),
            },
            SvgSeriesInput {
                label: "committed lines".to_owned(),
                values: fetched
                    .code_series
                    .iter()
                    .map(|b| b.commit_insertions)
                    .collect(),
                value_display: compact(committed),
            },
        ],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: compact,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(range.from + (range.to - range.from) / 2),
        x_end_display: date_label(range.to),
        show_area: false,
    })
}

fn model_pie(fetched: &AnalyticsUserData) -> PieView {
    let total: i64 = fetched.models.iter().map(|m| m.requests).sum();
    let slices = fetched
        .models
        .iter()
        .map(|m| PieSliceInput {
            label: m.model.clone(),
            value: m.requests,
            value_display: format!(
                "{} req · {} tok · {}",
                m.requests,
                compact(m.tokens),
                format_cost(m.cost_microdollars)
            ),
            filter_url: None,
        })
        .collect();
    pie_view(
        "Model mix",
        format!("{total} requests across {} models", fetched.models.len()),
        slices,
        "No model usage in this window.",
    )
}

fn code_totals(fetched: &AnalyticsUserData) -> UserCodeTotalsView {
    let t = &fetched.code_totals;
    UserCodeTotalsView {
        loc_added_ai_display: compact(t.loc_added_ai),
        loc_removed_ai_display: compact(t.loc_removed_ai),
        committed_display: format!(
            "+{} \u{2212}{}",
            compact(t.commit_insertions),
            compact(t.commit_deletions)
        ),
        commits_display: compact(t.commits),
        edit_operations_display: compact(t.ai_edit_operations),
    }
}

fn daily_rows(fetched: &AnalyticsUserData) -> Vec<UserDailyRowView> {
    fetched
        .daily
        .iter()
        .map(|r| UserDailyRowView {
            date_display: r.date.format("%Y-%m-%d").to_string(),
            sessions: i64::from(r.sessions_count),
            prompts: r.prompts,
            tool_uses: r.tool_uses,
            requests: r.ai_requests_count,
            loc_added_display: compact(r.loc_added_ai),
            commits: i64::from(r.commits_count),
            cost_display: format_cost(r.cost_microdollars),
        })
        .collect()
}

fn session_rows(fetched: &AnalyticsUserData) -> Vec<UserSessionRowView> {
    fetched
        .sessions
        .iter()
        .map(|r| UserSessionRowView {
            session_id: r.session_id.clone(),
            model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
            cost_display: format_cost(r.total_cost_microdollars),
            context_display: compact(r.context_window_size),
            cache_read_display: compact(r.cache_read_input_tokens),
            tokens_display: compact(r.input_tokens + r.output_tokens),
            updated_display: r.updated_at.format("%Y-%m-%d %H:%M").to_string(),
            session_url: format!(
                "/admin/entities/sessions/{}",
                urlencoding::encode(r.session_id.as_str())
            ),
        })
        .collect()
}

fn compact(v: i64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    } else if v >= 10_000 {
        format!("{}k", v / 1000)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

fn date_label(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&chrono::Local).format("%b %d").to_string()
}
