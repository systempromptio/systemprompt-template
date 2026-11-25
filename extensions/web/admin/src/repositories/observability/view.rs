//! The observability page's view: the declared exporter target and the
//! signals it ships. Pure — the handler reads, this maps, the template prints.

use serde::Serialize;
use systemprompt::manifest::profile::{OtlpSignal, OtlpTargetView};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExporterView {
    pub endpoint: String,
    pub protocol: &'static str,
    pub signals: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SignalView {
    pub signal: &'static str,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObservabilityView {
    pub configured: bool,
    pub exporter: Option<ExporterView>,
    pub signals: Vec<SignalView>,
    pub signal_count: usize,
}

#[must_use]
pub fn exporter_view(target: &OtlpTargetView) -> ExporterView {
    ExporterView {
        endpoint: target.endpoint.clone(),
        protocol: target.protocol.label(),
        signals: target.signals.iter().map(|s| s.label()).collect(),
    }
}

#[must_use]
pub fn build_view(target: Option<&OtlpTargetView>) -> ObservabilityView {
    let signals: Vec<SignalView> = OtlpSignal::ALL
        .into_iter()
        .map(|signal| SignalView {
            signal: signal.label(),
            enabled: target.is_some_and(|t| t.signals.contains(&signal)),
        })
        .collect();
    ObservabilityView {
        configured: target.is_some(),
        exporter: target.map(exporter_view),
        signal_count: signals.iter().filter(|s| s.enabled).count(),
        signals,
    }
}
