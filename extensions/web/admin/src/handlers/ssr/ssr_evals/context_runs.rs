//! View models for the rows an eval run produces: the run itself, each judged
//! result with its per-dimension scores, and the golden-set cases a run
//! replays. Split from `context` so neither file outgrows the size ceiling.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct RunRowView {
    pub id: String,
    pub short_id: String,
    pub kind: String,
    pub status: String,
    pub is_running: bool,
    pub is_failed: bool,
    pub judge_model: String,
    pub sample_size: i32,
    pub scored_count: i32,
    pub failed_count: i32,
    pub mean_score_display: String,
    pub cost_display: String,
    pub created_by: String,
    pub created_at_local: String,
    pub detail_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ResultRowView {
    pub id: String,
    pub run_id: String,
    pub ai_request_id: Option<String>,
    pub case_id: Option<String>,
    pub model: String,
    pub provider: String,
    pub score_display: String,
    pub score_pct: i64,
    pub verdict: String,
    pub is_pass: bool,
    pub is_partial: bool,
    pub is_fail: bool,
    pub rationale: String,
    pub flags: Vec<String>,
    pub has_flags: bool,
    pub dimensions: Vec<DimensionView>,
    pub prompt_excerpt: String,
    pub response_excerpt: String,
    pub latency_ms: Option<i32>,
    pub created_at_local: String,
    pub promote_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct DimensionView {
    pub label: &'static str,
    pub score: i64,
    pub pct: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct CaseRowView {
    pub id: String,
    pub name: String,
    pub baseline_model: String,
    pub expectation: String,
    pub has_expectation: bool,
    pub created_at_local: String,
}
