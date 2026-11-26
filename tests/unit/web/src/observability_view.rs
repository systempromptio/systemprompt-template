//! The observability page's pure mapping: the profile's credential-free OTLP
//! target into the view the template prints.

use systemprompt::manifest::profile::{OtlpProtocol, OtlpSignal, OtlpTargetView};
use systemprompt_web_admin::repositories::observability::view::{build_view, exporter_view};

fn target(signals: Vec<OtlpSignal>) -> OtlpTargetView {
    OtlpTargetView {
        endpoint: "https://otel.example.test".to_owned(),
        protocol: OtlpProtocol::Http,
        signals,
    }
}

#[test]
fn exporter_view_names_the_endpoint_protocol_and_signals() {
    let view = exporter_view(&target(vec![OtlpSignal::Traces]));
    assert_eq!(view.endpoint, "https://otel.example.test");
    assert_eq!(view.signals, vec!["traces"]);
    assert_eq!(view.protocol, "http");
}

#[test]
fn build_view_without_a_target_is_unconfigured_and_every_signal_is_off() {
    let view = build_view(None);
    assert!(!view.configured);
    assert!(view.exporter.is_none());
    assert_eq!(view.signal_count, 0);
    assert_eq!(view.signals.len(), OtlpSignal::ALL.len());
    assert!(view.signals.iter().all(|s| !s.enabled));
}

#[test]
fn build_view_lists_every_signal_and_enables_the_declared_ones() {
    let cfg = target(vec![OtlpSignal::Traces]);
    let view = build_view(Some(&cfg));
    assert!(view.configured);
    assert_eq!(view.signal_count, 1);
    let traces = view.signals.iter().find(|s| s.signal == "traces").unwrap();
    assert!(traces.enabled);
    let logs = view.signals.iter().find(|s| s.signal == "logs").unwrap();
    assert!(!logs.enabled);
}
