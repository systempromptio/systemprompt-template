//! The observability page's pure mapping: profile block plus
//! `otlp_export_state` rows into the view the template prints.

use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use systemprompt::models::profile::{OtlpExportConfig, OtlpProtocol, OtlpSignal};
use systemprompt::scheduler::OtlpExportState;
use systemprompt_web_admin::repositories::observability::view::{
    build_view, exporter_view, lag_label, lag_view, signal_status,
};

fn config(signals: Vec<OtlpSignal>) -> OtlpExportConfig {
    OtlpExportConfig {
        endpoint: "https://otel.example.test".to_owned(),
        protocol: OtlpProtocol::Http,
        headers: BTreeMap::from([("x-api-key".to_owned(), "secret-value".to_owned())]),
        signals,
        batch_seconds: 30,
    }
}

fn state(signal: &str, lag_seconds: i64, last_error: Option<&str>) -> OtlpExportState {
    let now = Utc::now();
    OtlpExportState {
        signal: signal.to_owned(),
        watermark: now - Duration::seconds(lag_seconds),
        watermark_id: "req_123456789".to_owned(),
        last_attempt_at: Some(now),
        last_success_at: last_error.is_none().then_some(now),
        last_error: last_error.map(str::to_owned),
        last_error_at: last_error.map(|_| now),
        batches_total: 4,
        failures_total: i64::from(last_error.is_some()),
        rows_total: 40,
        lag_seconds,
    }
}

#[test]
fn lag_label_picks_the_largest_unit() {
    assert_eq!(lag_label(-3), "ahead");
    assert_eq!(lag_label(12), "12s");
    assert_eq!(lag_label(125), "2m 5s");
    assert_eq!(lag_label(3660), "1h 1m");
    assert_eq!(lag_label(90_000), "1d 1h");
}

#[test]
fn lag_tone_steps_at_a_minute_and_an_hour() {
    assert_eq!(lag_view(5).tone, "ok");
    assert_eq!(lag_view(60).tone, "warn");
    assert_eq!(lag_view(3600).tone, "err");
}

#[test]
fn signal_status_orders_off_never_failing_healthy() {
    assert_eq!(signal_status(false, true, true).0, "disabled");
    assert_eq!(
        signal_status(true, false, false).0,
        "waiting for first tick"
    );
    assert_eq!(signal_status(true, true, true), ("failing", "err"));
    assert_eq!(signal_status(true, true, false), ("exporting", "ok"));
}

#[test]
fn exporter_view_names_headers_but_never_their_values() {
    let view = exporter_view(&config(vec![OtlpSignal::Traces]));
    assert_eq!(view.header_names, vec!["x-api-key".to_owned()]);
    assert_eq!(view.traces_url, "https://otel.example.test/v1/traces");
    assert_eq!(view.logs_url, "https://otel.example.test/v1/logs");
    assert_eq!(view.signals, vec!["traces"]);
    assert_eq!(view.protocol, "http");
    let serialized = serde_json::to_string(&view).unwrap();
    assert!(!serialized.contains("secret-value"));
}

#[test]
fn build_view_without_a_block_is_unconfigured_and_keeps_orphan_rows() {
    let states = vec![state("logs", 10, Some("collector answered 401"))];
    let view = build_view(None, &states);
    assert!(!view.configured);
    assert!(view.exporter.is_none());
    assert_eq!(view.signal_count, 1);
    let logs = &view.signals[0];
    assert_eq!(logs.signal, "logs");
    assert!(!logs.enabled);
    assert_eq!(logs.status, "disabled");
    assert!(logs.lag.is_none());
    assert_eq!(logs.last_error.as_deref(), Some("collector answered 401"));
    assert_eq!(view.totals.failures, 1);
    assert_eq!(view.totals.failures_tone, "warn");
}

#[test]
fn build_view_lists_every_declared_signal_even_before_its_first_tick() {
    let cfg = config(vec![OtlpSignal::Traces, OtlpSignal::Logs]);
    let states = vec![state("traces", 7, None)];
    let view = build_view(Some(&cfg), &states);
    assert!(view.configured);
    assert_eq!(view.signal_count, 2);
    let traces = &view.signals[0];
    assert_eq!(traces.status, "exporting");
    assert_eq!(traces.lag.as_ref().map(|l| l.seconds), Some(7));
    assert_eq!(traces.watermark_id.as_deref(), Some("req_123456789"));
    let logs = &view.signals[1];
    assert_eq!(logs.status, "waiting for first tick");
    assert!(logs.watermark_at.is_none());
    assert_eq!(logs.batches_total, 0);
    assert_eq!(view.totals.batches, 4);
    assert_eq!(view.totals.rows, 40);
    assert_eq!(view.totals.failures_tone, "ok");
}

#[test]
fn build_view_hides_an_empty_watermark_id() {
    let cfg = config(vec![OtlpSignal::Traces]);
    let mut caught_up = state("traces", 3, None);
    caught_up.watermark_id = String::new();
    let view = build_view(Some(&cfg), &[caught_up]);
    assert!(view.signals[0].watermark_id.is_none());
}
