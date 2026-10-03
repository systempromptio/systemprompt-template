//! Usage against ceiling, per subject, for every window the gateway is
//! enforcing right now.
//!
//! The gateway reserves against one bucket per `(subject, window)` — the
//! one whose `window_start` is the current aligned boundary — so that is
//! the one bucket read here; nothing is summed across periods. A subject
//! with no bucket has made no request in the window and is not listed.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::gateway::QuotaWindow;

use super::month_window::align_window;

#[derive(Debug, Clone, Serialize)]
pub struct SubjectUsage {
    pub subject_kind: String,
    pub subject_id: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_microdollars: i64,
    pub updated_at: DateTime<Utc>,
}

/// One window with the bucket every subject holds in it.
#[derive(Debug, Clone, Serialize)]
pub struct WindowUsage {
    pub window: QuotaWindow,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub subjects: Vec<SubjectUsage>,
}

pub async fn get_window_usage(
    pool: &PgPool,
    window: &QuotaWindow,
    now: DateTime<Utc>,
) -> Result<WindowUsage, sqlx::Error> {
    let window_start = align_window(now, window.window_seconds);
    let window_end =
        window_start + chrono::Duration::seconds(i64::from(window.window_seconds.max(1)));
    let subjects = sqlx::query_as!(
        SubjectUsage,
        r"SELECT subject_kind, subject_id, requests, input_tokens, output_tokens,
                 cost_microdollars, updated_at
            FROM ai_quota_buckets
           WHERE subject_kind = $1 AND window_seconds = $2 AND window_start = $3
           ORDER BY cost_microdollars DESC, requests DESC, subject_id ASC
           LIMIT 500",
        window.subject,
        window.window_seconds,
        window_start
    )
    .fetch_all(pool)
    .await?;
    Ok(WindowUsage {
        window: window.clone(),
        window_start,
        window_end,
        subjects,
    })
}

/// How far a bucket is into its ceilings: the largest fraction across the
/// ceilings the window sets, in percent, and which ceiling it is.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Headroom {
    pub percent: u32,
    pub ceiling: &'static str,
    pub breached: bool,
}

#[must_use]
pub fn headroom(window: &QuotaWindow, usage: &SubjectUsage) -> Headroom {
    let candidates = [
        (window.max_requests, usage.requests, "requests"),
        (window.max_input_tokens, usage.input_tokens, "input tokens"),
        (
            window.max_output_tokens,
            usage.output_tokens,
            "output tokens",
        ),
        (
            window.max_cost_microdollars,
            usage.cost_microdollars,
            "cost",
        ),
    ];
    let mut best = Headroom {
        percent: 0,
        ceiling: "none",
        breached: false,
    };
    for (max, used, label) in candidates {
        let Some(max) = max.filter(|m| *m > 0) else {
            continue;
        };
        let percent = (used.max(0).saturating_mul(100) / max).min(u32::MAX.into());
        let percent = u32::try_from(percent).unwrap_or(u32::MAX);
        if percent >= best.percent {
            best = Headroom {
                percent,
                ceiling: label,
                breached: used > max,
            };
        }
    }
    best
}
