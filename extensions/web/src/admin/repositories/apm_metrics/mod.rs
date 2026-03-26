mod calculations;
mod hourly;
mod performance;
mod queries;
mod throughput;

pub use calculations::*;
pub use hourly::*;
pub use performance::*;
pub use queries::*;
pub use throughput::*;

use crate::admin::numeric;

#[derive(Debug, Default, Clone)]
pub struct TodayApmLive {
    pub current_apm: f32,
    pub peak_apm: f32,
    pub avg_apm: f32,
    pub current_concurrency: i32,
    pub peak_concurrency: i32,
    pub avg_concurrency: f32,
    pub total_throughput_display: String,
    pub throughput_rate_display: String,
    pub tool_diversity: i32,
    pub multitasking_score: f32,
}

#[derive(Debug, Default, Clone)]
pub struct ApmCorrelation {
    pub high_apm_success_rate: f32,
    pub medium_apm_success_rate: f32,
    pub low_apm_success_rate: f32,
    pub high_apm_avg_quality: f32,
    pub low_apm_avg_quality: f32,
}

#[derive(Debug, Default, Clone)]
pub struct HourlyApmBucket {
    pub hour: i32,
    pub actions: i64,
    pub errors: i64,
    pub sessions: i64,
    pub input_bytes: i64,
    pub output_bytes: i64,
    pub unique_tools: i64,
    pub subagent_spawns: i64,
}

#[derive(Debug, Default, Clone)]
pub struct DailyApmBucket {
    pub date: String,
    pub actions: i64,
    pub errors: i64,
    pub sessions: i64,
    pub input_bytes: i64,
    pub output_bytes: i64,
    pub unique_tools: i64,
    pub subagent_spawns: i64,
}

#[derive(Debug, Default, Clone)]
pub struct TodayPerformanceSummary {
    pub total_sessions: i64,
    pub total_actions: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub error_rate_pct: f32,
    pub total_input_bytes: i64,
    pub total_output_bytes: i64,
    pub avg_apm: f32,
    pub peak_apm: f32,
    pub peak_concurrency: i32,
    pub tool_diversity: i32,
    pub multitasking_score: f32,
    pub active_minutes: f32,
}

#[must_use]
pub fn format_bytes_rate(bytes: i64, seconds: f64) -> String {
    if seconds <= 0.0 {
        return "0 B/s".to_string();
    }
    let bps = numeric::to_f64(bytes) / seconds;
    if bps < 1_024.0 {
        format!("{bps:.0} B/s")
    } else if bps < 1_048_576.0 {
        format!("{:.1} KB/s", bps / 1_024.0)
    } else if bps < 1_073_741_824.0 {
        format!("{:.1} MB/s", bps / 1_048_576.0)
    } else {
        format!("{:.1} GB/s", bps / 1_073_741_824.0)
    }
}

#[must_use]
pub fn format_total_bytes(total_bytes: i64) -> String {
    if total_bytes < 1024 {
        format!("{total_bytes} B")
    } else if total_bytes < 1_048_576 {
        format!("{:.1} KB", numeric::to_f64(total_bytes) / 1024.0)
    } else {
        format!("{:.1} MB", numeric::to_f64(total_bytes) / 1_048_576.0)
    }
}
