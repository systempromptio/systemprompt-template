//! Step-by-step outcome of a connection probe, returned to the browser so the
//! Connectors page can show which stage of the handshake passed or failed.

use crate::error::AdminResult;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct VerificationStep {
    pub step: &'static str,
    pub ok: bool,
    pub detail: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct VerificationReport {
    pub provider: String,
    pub ok: bool,
    pub steps: Vec<VerificationStep>,
    pub error: Option<String>,
}

impl VerificationReport {
    pub fn for_provider(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            ..Self::default()
        }
    }

    pub fn record(&mut self, step: &'static str, started: Instant, outcome: &AdminResult<String>) {
        let (ok, detail) = match outcome {
            Ok(detail) => (true, detail.clone()),
            Err(error) => (false, error.to_string()),
        };
        if !ok {
            self.error = Some(detail.clone());
        }
        self.steps.push(VerificationStep {
            step,
            ok,
            detail,
            duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        });
    }

    pub const fn finish(mut self) -> Self {
        self.ok = self.error.is_none() && !self.steps.is_empty();
        self
    }
}
