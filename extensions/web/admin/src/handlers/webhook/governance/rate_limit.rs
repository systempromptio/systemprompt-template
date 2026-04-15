use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Mutex;
use std::time::Instant;

use systemprompt::identifiers::{SessionId, UserId};

const WINDOW_SECS: u64 = 60;
const MAX_PER_WINDOW: usize = 300;

static COUNTERS: Mutex<Option<SlidingWindow>> = Mutex::new(None);

struct SlidingWindow {
    buckets: HashMap<String, Vec<Instant>>,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    fn check_and_record(&mut self, key: &str) -> usize {
        let now = Instant::now();
        let cutoff = now
            .checked_sub(std::time::Duration::from_secs(WINDOW_SECS))
            .unwrap_or(now);

        let timestamps = self.buckets.entry(key.to_string()).or_default();
        timestamps.retain(|t| *t > cutoff);
        let count = timestamps.len();

        if count < MAX_PER_WINDOW {
            timestamps.push(now);
        }

        count
    }
}

pub(super) fn check(session_id: &SessionId, user_id: &UserId) -> (usize, usize) {
    let mut key = String::with_capacity(64);
    write!(key, "{}:{}", session_id.as_str(), user_id.as_str()).ok();
    let count = COUNTERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_with(SlidingWindow::new)
        .check_and_record(&key);
    (count, MAX_PER_WINDOW)
}
