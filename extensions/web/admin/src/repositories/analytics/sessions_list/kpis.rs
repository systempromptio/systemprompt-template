//! Stat-strip totals for the sessions list.
//!
//! Aggregates the same `FULL OUTER JOIN` the list query pages over, under the
//! same filter, so the strip always describes the rows below it rather than
//! the whole table.


#[derive(Debug, Clone, Copy, Default)]
pub struct SessionListKpis {
    pub total_sessions: i64,
    pub error_sessions: i64,
    pub total_requests: i64,
    pub total_tool_uses: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
}
