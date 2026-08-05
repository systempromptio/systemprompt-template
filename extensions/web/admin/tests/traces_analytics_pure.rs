#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use systemprompt_web_admin::repositories::analytics::contexts_list::ContextListFilter;
use systemprompt_web_admin::repositories::analytics::conversations::ConversationListFilter;
use systemprompt_web_admin::repositories::analytics::requests::{
    RequestFilter, RequestSortColumn, RequestSortSpec, SortDir,
};
use systemprompt_web_admin::repositories::traces::{
    SpanKind, SpanStatus, TraceFilter, TraceSort, TraceSortColumn, TraceSortDir, TraceStats,
};

#[test]
fn span_kind_serialises_to_its_snake_case_name() {
    let cases = [
        (SpanKind::Gateway, "gateway"),
        (SpanKind::Governance, "governance"),
        (SpanKind::Tool, "tool"),
        (SpanKind::Model, "model"),
        (SpanKind::Spawn, "spawn"),
    ];
    for (kind, expected) in cases {
        assert_eq!(kind.as_str(), expected);
        assert_eq!(
            serde_json::to_value(kind).expect("serialize span kind"),
            serde_json::Value::String(expected.to_owned()),
            "as_str and the wire form must not drift"
        );
    }
}

#[test]
fn span_status_serialises_to_its_snake_case_name() {
    let cases = [
        (SpanStatus::Ok, "ok"),
        (SpanStatus::Deny, "deny"),
        (SpanStatus::Error, "error"),
        (SpanStatus::Pending, "pending"),
    ];
    for (status, expected) in cases {
        assert_eq!(status.as_str(), expected);
        assert_eq!(
            serde_json::to_value(status).expect("serialize span status"),
            serde_json::Value::String(expected.to_owned()),
        );
    }
}

#[test]
fn trace_sort_defaults_to_newest_first() {
    let sort = TraceSort::default();
    assert!(matches!(sort.column, TraceSortColumn::StartedAt));
    assert!(matches!(sort.dir, TraceSortDir::Desc));
}

#[test]
fn trace_filter_defaults_to_no_narrowing() {
    let filter = TraceFilter::default();
    assert!(filter.user_id.is_none());
    assert!(filter.agent_id.is_none());
    assert!(filter.agent_scope.is_none());
    assert!(filter.policy.is_none());
    assert!(filter.decision.is_none());
    assert!(!filter.error_only);
    assert!(!filter.deny_only);
}

#[test]
fn trace_stats_default_to_zero() {
    let stats = TraceStats::default();
    assert_eq!(stats.total_traces, 0);
    assert_eq!(stats.error_count, 0);
    assert_eq!(stats.deny_count, 0);
    assert_eq!(stats.p50_active_ms, 0);
    assert_eq!(stats.p95_active_ms, 0);
    assert_eq!(stats.p99_active_ms, 0);
    assert_eq!(stats.total_cost_microdollars, 0);
    assert_eq!(stats.total_tokens, 0);
}

#[test]
fn request_sort_defaults_to_newest_first() {
    let spec = RequestSortSpec::default();
    assert!(matches!(spec.column, RequestSortColumn::CreatedAt));
    assert!(matches!(spec.dir, SortDir::Desc));
}

#[test]
fn request_filter_defaults_to_no_narrowing() {
    let filter = RequestFilter::default();
    assert!(filter.user_id.is_none());
    assert!(filter.agent_id.is_none());
    assert!(filter.model.is_none());
    assert!(filter.provider.is_none());
    assert!(filter.status.is_none());
    assert!(filter.search.is_none());
}

#[test]
fn context_list_filter_defaults_to_no_narrowing() {
    let filter = ContextListFilter::default();
    assert!(filter.user_id.is_none());
    assert!(filter.model.is_none());
    assert!(filter.free_text.is_none());
    assert!(filter.since.is_none());
    assert_eq!(filter.limit, 0);
}

#[test]
fn conversation_filter_defaults_to_an_unbounded_window() {
    let filter = ConversationListFilter::default();
    assert!(filter.user_id.is_none());
    assert!(filter.plugin_id.is_none());
    assert!(filter.free_text.is_none());
    assert!(filter.since.is_none());
    assert!(filter.until.is_none());
    assert_eq!(filter.limit, 0);
}
