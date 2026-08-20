//! Rollups derived from hook events as sessions progress.

mod ai_summaries;
mod daily;
pub(crate) mod session_updates;
mod statusline;

pub use ai_summaries::update_session_ai_summary_structured;
pub use daily::{
    DailyAggregationParams, SessionSummaryParams, increment_session_summary,
    upsert_daily_aggregation,
};
pub use session_updates::{
    update_session_metadata, update_session_permission_mode, update_session_title_if_empty,
};
pub use statusline::{
    SessionCostSnapshot, set_session_summary_tokens, upsert_session_cost_snapshot,
};
