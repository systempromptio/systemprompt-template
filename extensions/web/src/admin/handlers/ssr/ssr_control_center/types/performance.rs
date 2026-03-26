use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct TodaySummaryObj {
    pub sessions_count: i64,
    pub analysed_count: i64,
    pub avg_quality: String,
    pub goals_achieved: i64,
    pub goals_partial: i64,
    pub goals_failed: i64,
    pub new_achievements: Vec<String>,
    pub has_new_achievements: bool,
    pub top_recommendation: String,
    pub has_top_recommendation: bool,
}

#[derive(Serialize, Clone)]
pub struct HourlyEntry {
    pub hour: i32,
    pub actions: i64,
    pub errors: i64,
    pub sessions: i64,
    pub input_bytes: i64,
    pub output_bytes: i64,
    pub unique_tools: i64,
    pub subagent_spawns: i64,
}

#[derive(Serialize, Clone)]
pub struct PerfSummaryObj {
    pub total_sessions: i64,
    pub total_actions: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub error_rate_pct: String,
    pub total_input_bytes: i64,
    pub total_output_bytes: i64,
    pub avg_apm: String,
    pub peak_apm: String,
    pub peak_concurrency: i32,
    pub tool_diversity: i32,
    pub multitasking_score: String,
    pub active_minutes: String,
}

#[derive(Serialize, Clone)]
pub struct ApmMetrics {
    pub apm: ApmValues,
    pub concurrency: ConcurrencyValues,
    pub throughput: ThroughputValues,
    pub tool_diversity: i32,
    pub multitasking_score: f32,
}

#[derive(Serialize, Clone)]
pub struct ApmValues {
    pub current: f32,
    pub peak: f32,
    pub avg: f32,
}

#[derive(Serialize, Clone)]
pub struct ConcurrencyValues {
    pub current: i32,
    pub peak: i32,
    pub avg: f32,
}

#[derive(Serialize, Clone)]
pub struct ThroughputValues {
    pub total_display: String,
    pub rate_display: String,
}

#[derive(Serialize, Clone)]
pub struct InitialChartData {
    pub hourly: Vec<HourlyEntry>,
    pub perf: PerfSummaryObj,
    pub apm_metrics: ApmMetrics,
}

#[derive(Serialize, Clone)]
pub struct TodayObj {
    pub active_now: usize,
    pub completed: i64,
    pub success_rate: i64,
    pub has_success_rate: bool,
    pub sessions: i64,
    pub prompts: i64,
    pub tool_calls: i64,
    pub errors: i64,
    pub total_content_display: String,
    pub apm_current: String,
    pub apm_peak: String,
    pub eapm_current: String,
    pub concurrency_current: i32,
    pub concurrency_peak: i32,
    pub throughput_total: String,
    pub tool_diversity: i32,
    pub multitasking_score: f32,
}
