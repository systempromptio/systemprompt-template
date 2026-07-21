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
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{PolicyId, SessionId, UserId};
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{GovernancePolicy, PolicyContext, RateLimitWindow};

use super::super::policy::PolicyRegistration;

const ID: &str = "rate_limit";
const DEFAULT_WINDOW_SECS: u64 = 60;
const DEFAULT_LIMIT: usize = 300;

#[derive(Debug)]
pub(super) struct RateLimit {
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

        let timestamps = self.buckets.entry(key.to_owned()).or_default();
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
    format!("{}:{}", session_id.as_str(), user_id.as_str())
}

impl GovernancePolicy for RateLimit {
    fn id(&self) -> PolicyId {
        PolicyId::new(ID)
    }
    fn name(&self) -> &'static str {
        "Rate Limit"
    }
    fn description(&self) -> &'static str {
        "Sliding-window per-session per-user request limiter. Stops a single \
         caller from monopolising the gateway or exfiltrating data via volume."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> Decision {
        let key = key_for(ctx.session_id, ctx.user_id);
        let count = COUNTERS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .check_and_record(&key, self.window_secs, self.limit);

        let window = RateLimitWindow {
            name: ID.to_owned(),
            seconds: self.window_secs,
            limit: self.limit as u64,
        };

        if count >= self.limit {
            Decision::Deny {
                reason: DenyReason::RateLimitExceeded {
                    window,
                    retry_after_ms: self.window_secs.saturating_mul(1000),
                },
            }
        } else {
            Decision::Allow {
                matched_by: MatchedBy::PolicyAllow {
                    policy_id: PolicyId::new(ID),
                    detail: Cow::Owned(format!(
                        "{count}/{} calls in {}s window",
                        self.limit, self.window_secs
                    )),
                },
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
