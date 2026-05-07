//! `rate_limit`: per-`{session,user}` sliding-window limiter.
//!
//! Configurable via:
//! ```yaml
//! - id: rate_limit
//!   requests_per_window: 300
//!   window_secs: 60
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{SessionId, UserId};

use super::super::policy::{Policy, PolicyContext, PolicyOutcome, PolicyRegistration};

const ID: &str = "rate_limit";
const DEFAULT_WINDOW_SECS: u64 = 60;
const DEFAULT_LIMIT: usize = 300;

pub struct RateLimit {
    window_secs: u64,
    limit: usize,
}

impl RateLimit {
    fn from_yaml(v: &YamlValue) -> Self {
        let window_secs = v
            .get("window_secs")
            .and_then(YamlValue::as_u64)
            .unwrap_or(DEFAULT_WINDOW_SECS);
        let limit = v
            .get("requests_per_window")
            .and_then(YamlValue::as_u64)
            .map_or(DEFAULT_LIMIT, |n| n as usize);
        Self { window_secs, limit }
    }
}

#[derive(Default)]
struct SlidingWindow {
    buckets: HashMap<String, Vec<Instant>>,
}

impl SlidingWindow {
    fn check_and_record(&mut self, key: &str, window_secs: u64, limit: usize) -> usize {
        let now = Instant::now();
        let cutoff = now
            .checked_sub(Duration::from_secs(window_secs))
            .unwrap_or(now);

        let timestamps = self.buckets.entry(key.to_string()).or_default();
        timestamps.retain(|t| *t > cutoff);
        let count = timestamps.len();

        if count < limit {
            timestamps.push(now);
        }

        count
    }
}

static COUNTERS: LazyLock<Mutex<SlidingWindow>> =
    LazyLock::new(|| Mutex::new(SlidingWindow::default()));

fn key_for(session_id: &SessionId, user_id: &UserId) -> String {
    let mut k = String::with_capacity(64);
    write!(k, "{}:{}", session_id.as_str(), user_id.as_str()).ok();
    k
}

impl Policy for RateLimit {
    fn id(&self) -> &'static str {
        ID
    }
    fn name(&self) -> &'static str {
        "Rate Limit"
    }
    fn description(&self) -> &'static str {
        "Sliding-window per-session per-user request limiter. Stops a single \
         caller from monopolising the gateway or exfiltrating data via volume."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyOutcome {
        let key = key_for(ctx.session_id, ctx.user_id);
        let count = COUNTERS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .check_and_record(&key, self.window_secs, self.limit);

        if count >= self.limit {
            PolicyOutcome::Deny {
                reason: format!(
                    "Rate limit exceeded: {count}/{} calls in {}s window",
                    self.limit, self.window_secs
                ),
                detail: Cow::Owned(format!(
                    "{count}/{} calls in {}s window — limit exceeded",
                    self.limit, self.window_secs
                )),
            }
        } else {
            PolicyOutcome::Allow {
                detail: Cow::Owned(format!(
                    "{count}/{} calls in {}s window",
                    self.limit, self.window_secs
                )),
            }
        }
    }
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(RateLimit::from_yaml(v)),
        source_path: file!(),
    }
}
