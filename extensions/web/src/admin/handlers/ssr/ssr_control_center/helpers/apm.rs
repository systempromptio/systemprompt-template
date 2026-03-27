use crate::admin::numeric;
use crate::admin::repositories::control_center;

use super::super::types::{
    ApmMetrics, ApmValues, ConcurrencyValues, HourlyEntry, InitialChartData, PerfSummaryObj,
    ThroughputValues, TodayObj, TodaySummaryObj,
};
use super::BuildTemplateDataParams;

pub(in crate::admin) struct ApmData {
    pub today_obj: TodayObj,
    pub today_summary_obj: TodaySummaryObj,
    pub hourly_json: Vec<HourlyEntry>,
    pub perf_json: PerfSummaryObj,
    pub cc_initial_json: String,
}

pub(in crate::admin) fn build_apm_data(params: &BuildTemplateDataParams<'_>) -> ApmData {
    let ts = params.today_summary;
    let today_summary_obj = TodaySummaryObj {
        sessions_count: ts.sessions_count,
        analysed_count: ts.analysed_count,
        avg_quality: format!("{:.1}", ts.avg_quality),
        goals_achieved: ts.goals_achieved,
        goals_partial: ts.goals_partial,
        goals_failed: ts.goals_failed,
        new_achievements: ts.new_achievements.clone(),
        has_new_achievements: !ts.new_achievements.is_empty(),
        top_recommendation: ts.top_recommendation.clone(),
        has_top_recommendation: !ts.top_recommendation.is_empty(),
    };

    let hourly_json: Vec<HourlyEntry> = params
        .hourly_breakdown
        .iter()
        .map(|b| HourlyEntry {
            hour: b.hour,
            actions: b.actions,
            errors: b.errors,
            sessions: b.sessions,
            input_bytes: b.input_bytes,
            output_bytes: b.output_bytes,
            unique_tools: b.unique_tools,
            subagent_spawns: b.subagent_spawns,
        })
        .collect();

    let perf_json = PerfSummaryObj {
        total_sessions: params.perf_summary.total_sessions,
        total_actions: params.perf_summary.total_actions,
        total_prompts: params.perf_summary.total_prompts,
        total_tool_uses: params.perf_summary.total_tool_uses,
        total_errors: params.perf_summary.total_errors,
        error_rate_pct: format!("{:.1}", params.perf_summary.error_rate_pct),
        total_input_bytes: params.perf_summary.total_input_bytes,
        total_output_bytes: params.perf_summary.total_output_bytes,
        avg_apm: format!("{:.1}", params.perf_summary.avg_apm),
        peak_apm: format!("{:.1}", params.perf_summary.peak_apm),
        peak_concurrency: params.perf_summary.peak_concurrency,
        tool_diversity: params.perf_summary.tool_diversity,
        multitasking_score: format!("{:.1}", params.perf_summary.multitasking_score),
        active_minutes: format!("{:.0}", params.perf_summary.active_minutes),
    };

    let today_obj = build_today_obj(params);

    let apm = params.apm_live;
    let initial_chart_data = InitialChartData {
        hourly: hourly_json.clone(),
        perf: perf_json.clone(),
        apm_metrics: ApmMetrics {
            apm: ApmValues {
                current: apm.current_apm,
                peak: apm.peak_apm,
                avg: apm.avg_apm,
            },
            concurrency: ConcurrencyValues {
                current: apm.current_concurrency,
                peak: apm.peak_concurrency,
                avg: apm.avg_concurrency,
            },
            throughput: ThroughputValues {
                total_display: apm.total_throughput_display.clone(),
                rate_display: apm.throughput_rate_display.clone(),
            },
            tool_diversity: apm.tool_diversity,
            multitasking_score: apm.multitasking_score,
        },
    };
    let cc_initial_json = serde_json::to_string(&initial_chart_data).unwrap_or_else(|_| String::new());

    ApmData {
        today_obj,
        today_summary_obj,
        hourly_json,
        perf_json,
        cc_initial_json,
    }
}

fn build_today_obj(params: &BuildTemplateDataParams<'_>) -> TodayObj {
    let success_rate = if params.outcome_stats.rated_count > 0 {
        numeric::to_i64(
            numeric::to_f64(params.outcome_stats.positive_count)
                / numeric::to_f64(params.outcome_stats.rated_count)
                * 100.0,
        )
    } else {
        0
    };

    TodayObj {
        active_now: params
            .session_groups
            .iter()
            .filter(|g| g.flags.is_active)
            .count(),
        completed: params.outcome_stats.completed_today,
        success_rate,
        has_success_rate: params.outcome_stats.rated_count > 0,
        sessions: params.today_stats.sessions_started,
        prompts: params.today_stats.total_prompts,
        tool_calls: params.today_stats.total_tool_calls,
        errors: params.today_stats.total_errors,
        total_content_display: control_center::format_bytes(
            params.today_stats.content_input_bytes + params.today_stats.content_output_bytes,
        ),
        apm_current: format!("{:.1}", params.apm_live.current_apm),
        apm_peak: format!("{:.1}", params.apm_live.peak_apm),
        eapm_current: format!("{:.1}", params.apm_live.avg_apm),
        concurrency_current: params.apm_live.current_concurrency,
        concurrency_peak: params.apm_live.peak_concurrency,
        throughput_total: params.apm_live.total_throughput_display.clone(),
        tool_diversity: params.apm_live.tool_diversity,
        multitasking_score: params.apm_live.multitasking_score,
    }
}
