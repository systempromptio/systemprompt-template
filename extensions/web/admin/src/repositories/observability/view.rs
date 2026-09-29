//! The observability page's view: the declared exporter and one row per
//! signal, derived from the profile block and the `otlp_export_state` rows.
//!
//! Pure — the handler reads, this maps, the template prints. A signal the
//! profile names but the table lacks is "waiting for its first tick"; a row
//! the table holds for a signal the profile no longer names is "disabled",
//! kept visible so its last error is not lost when someone turns it off.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::models::profile::{OtlpExportConfig, OtlpSignal};
use systemprompt::scheduler::OtlpExportState;

// Why: a lag under the job's own cadence is the exporter idling behind the
// settle window; past a minute it is falling behind; past an hour the
// collector has been unreachable for a while.
const LAG_WARN_SECONDS: i64 = 60;
const LAG_ERR_SECONDS: i64 = 3600;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExporterView {
    pub endpoint: String,
    pub protocol: &'static str,
    pub batch_seconds: u64,
    pub signals: Vec<&'static str>,
    // Why: header names only. The values are the collector's credentials
    // and never reach a page.
    pub header_names: Vec<String>,
    pub traces_url: String,
    pub logs_url: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LagView {
    pub seconds: i64,
    pub label: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SignalView {
    pub signal: &'static str,
    pub enabled: bool,
    pub status: &'static str,
    pub status_tone: &'static str,
    pub watermark_at: Option<String>,
    pub watermark_id: Option<String>,
    pub lag: Option<LagView>,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
    pub last_error_at: Option<String>,
    pub batches_total: i64,
    pub failures_total: i64,
    pub rows_total: i64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
pub struct TotalsView {
    pub batches: i64,
    pub failures: i64,
    pub rows: i64,
    pub failures_tone: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObservabilityView {
    pub configured: bool,
    pub exporter: Option<ExporterView>,
    pub signals: Vec<SignalView>,
    pub signal_count: usize,
    pub totals: TotalsView,
}

#[must_use]
pub fn exporter_view(config: &OtlpExportConfig) -> ExporterView {
    ExporterView {
        endpoint: config.endpoint.clone(),
        protocol: config.protocol.label(),
        batch_seconds: config.batch_seconds,
        signals: config.signals.iter().map(|s| s.label()).collect(),
        header_names: config.headers.keys().cloned().collect(),
        traces_url: config.signal_url(OtlpSignal::Traces),
        logs_url: config.signal_url(OtlpSignal::Logs),
    }
}

#[must_use]
pub fn lag_label(seconds: i64) -> String {
    match seconds {
        s if s < 0 => "ahead".to_owned(),
        s if s < 60 => format!("{s}s"),
        s if s < 3600 => format!("{}m {}s", s / 60, s % 60),
        s if s < 86_400 => format!("{}h {}m", s / 3600, (s % 3600) / 60),
        s => format!("{}d {}h", s / 86_400, (s % 86_400) / 3600),
    }
}

#[must_use]
pub fn lag_view(seconds: i64) -> LagView {
    let tone = if seconds >= LAG_ERR_SECONDS {
        "err"
    } else if seconds >= LAG_WARN_SECONDS {
        "warn"
    } else {
        "ok"
    };
    LagView {
        seconds,
        label: lag_label(seconds),
        tone,
    }
}

// Why: one word for the row, in the order an operator cares: off, never
// ran, failing, healthy.
#[must_use]
pub const fn signal_status(
    enabled: bool,
    has_state: bool,
    last_error: bool,
) -> (&'static str, &'static str) {
    match (enabled, has_state, last_error) {
        (false, _, _) => ("disabled", "muted"),
        (true, false, _) => ("waiting for first tick", "warn"),
        (true, true, true) => ("failing", "err"),
        (true, true, false) => ("exporting", "ok"),
    }
}

fn rfc3339(at: Option<DateTime<Utc>>) -> Option<String> {
    at.map(|t| t.to_rfc3339())
}

fn signal_view(signal: OtlpSignal, enabled: bool, state: Option<&OtlpExportState>) -> SignalView {
    let (status, status_tone) = signal_status(
        enabled,
        state.is_some(),
        state.is_some_and(|s| s.last_error.is_some()),
    );
    SignalView {
        signal: signal.label(),
        enabled,
        status,
        status_tone,
        watermark_at: rfc3339(state.map(|s| s.watermark)),
        watermark_id: state
            .map(|s| s.watermark_id.clone())
            .filter(|id| !id.is_empty()),
        lag: state.filter(|_| enabled).map(|s| lag_view(s.lag_seconds)),
        last_attempt_at: rfc3339(state.and_then(|s| s.last_attempt_at)),
        last_success_at: rfc3339(state.and_then(|s| s.last_success_at)),
        last_error: state.and_then(|s| s.last_error.clone()),
        last_error_at: rfc3339(state.and_then(|s| s.last_error_at)),
        batches_total: state.map_or(0, |s| s.batches_total),
        failures_total: state.map_or(0, |s| s.failures_total),
        rows_total: state.map_or(0, |s| s.rows_total),
    }
}

#[must_use]
pub fn build_view(
    config: Option<&OtlpExportConfig>,
    states: &[OtlpExportState],
) -> ObservabilityView {
    let signals: Vec<SignalView> = OtlpSignal::ALL
        .into_iter()
        .filter_map(|signal| {
            let enabled = config.is_some_and(|c| c.exports(signal));
            let state = states.iter().find(|s| s.signal == signal.label());
            (enabled || state.is_some()).then(|| signal_view(signal, enabled, state))
        })
        .collect();
    let mut totals = signals
        .iter()
        .fold(TotalsView::default(), |acc, s| TotalsView {
            batches: acc.batches + s.batches_total,
            failures: acc.failures + s.failures_total,
            rows: acc.rows + s.rows_total,
            failures_tone: "",
        });
    totals.failures_tone = if totals.failures > 0 { "warn" } else { "ok" };
    ObservabilityView {
        configured: config.is_some(),
        exporter: config.map(exporter_view),
        signal_count: signals.len(),
        signals,
        totals,
    }
}
