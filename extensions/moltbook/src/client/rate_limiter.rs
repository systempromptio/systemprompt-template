use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::MoltbookError;

#[derive(Debug)]
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limits: HashMap<String, RateLimit>,
}

#[derive(Debug, Clone)]
struct RateLimit {
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();

        limits.insert(
            "post".to_string(),
            RateLimit {
                max_requests: 1,
                window: Duration::from_secs(30 * 60),
            },
        );

        limits.insert(
            "comment".to_string(),
            RateLimit {
                max_requests: 50,
                window: Duration::from_secs(60 * 60),
            },
        );

        limits.insert(
            "read".to_string(),
            RateLimit {
                max_requests: 100,
                window: Duration::from_secs(60),
            },
        );

        limits.insert(
            "vote".to_string(),
            RateLimit {
                max_requests: 60,
                window: Duration::from_secs(60),
            },
        );

        Self {
            requests: HashMap::new(),
            limits,
        }
    }

    pub fn check_and_update(&mut self, operation: &str) -> Result<(), MoltbookError> {
        let limit = self.limits.get(operation).cloned().unwrap_or(RateLimit {
            max_requests: 100,
            window: Duration::from_secs(60),
        });

        let now = Instant::now();
        let cutoff = now - limit.window;

        let requests = self.requests.entry(operation.to_string()).or_default();

        requests.retain(|&t| t > cutoff);

        if requests.len() >= limit.max_requests {
            let oldest = requests.first().copied().unwrap_or(now);
            let wait_time = limit.window - (now - oldest);

            return Err(MoltbookError::RateLimitExceeded(format!(
                "Rate limit exceeded for '{}'. {} requests in {:?}. Wait {:?}.",
                operation, limit.max_requests, limit.window, wait_time
            )));
        }

        requests.push(now);
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
