//! Seats-tab view types for the analytics dashboard.
//!
//! Split from `context.rs` at the 300-line ceiling, on the seam the tab
//! already draws: seat utilisation, the wasted-seat rows, and the inactivity
//! window that decides which members appear in them.

use serde::Serialize;
use systemprompt::identifiers::UserId;

#[derive(Debug, Serialize)]
pub(super) struct InactiveDayOption {
    pub days: i32,
    pub label: String,
    pub href: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct SeatSummaryView {
    pub org_name: String,
    pub seats_used: i64,
    pub seat_limit_display: String,
    pub pct: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct WastedSeatView {
    pub user_id: UserId,
    pub label: String,
    pub email: String,
    pub department: String,
    pub org_name: String,
    pub last_request_display: String,
    pub detail_url: String,
    pub analytics_url: String,
}
