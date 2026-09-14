#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

//! `/hooks/statusline` ingest rules: how a Claude Code statusline document
//! becomes the one validated record the handler stores. These pin session
//! resolution (query wins, body is the fallback, neither is a rejection),
//! the microdollar conversion, and every range rule — each rejection names
//! the field at fault.

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::types::webhook::{
    StatusLineIngest, StatusLinePayload, StatusLineQuery, StatusLineRejection, usd_to_microdollars,
};

fn query(session: Option<&str>) -> StatusLineQuery {
    serde_json::from_value(serde_json::json!({ "session_id": session })).expect("query parses")
}

fn payload(json: serde_json::Value) -> StatusLinePayload {
    serde_json::from_value(json).expect("payload parses")
}

fn ingest(
    session: Option<&str>,
    json: serde_json::Value,
) -> Result<StatusLineIngest, StatusLineRejection> {
    StatusLineIngest::try_from((query(session), payload(json)))
}

#[test]
fn the_body_session_is_used_when_the_query_names_none() {
    let record = ingest(None, serde_json::json!({ "session_id": "body-session" }))
        .expect("body session identifies the request");
    assert_eq!(record.session_id, SessionId::new("body-session"));
}

#[test]
fn agreeing_query_and_body_sessions_identify_the_request_once() {
    let record = ingest(Some("s-1"), serde_json::json!({ "session_id": "s-1" }))
        .expect("the same session named twice is not a conflict");
    assert_eq!(record.session_id, SessionId::new("s-1"));
}

#[test]
fn a_query_session_that_disagrees_with_the_body_is_rejected() {
    assert_eq!(
        ingest(
            Some("query-session"),
            serde_json::json!({ "session_id": "body-session" }),
        ),
        Err(StatusLineRejection::SessionMismatch)
    );
}

#[test]
fn a_request_naming_no_session_anywhere_is_rejected() {
    assert_eq!(
        ingest(
            None,
            serde_json::json!({ "cost": { "total_cost_usd": 0.42 } })
        ),
        Err(StatusLineRejection::MissingSession)
    );
}

#[test]
fn a_malformed_session_key_is_rejected_by_name() {
    let rejection = ingest(None, serde_json::json!({ "session_id": "has space" }))
        .expect_err("spaces are not identifier bytes");
    assert!(matches!(rejection, StatusLineRejection::InvalidSession(_)));
    assert!(rejection.to_string().contains("session_id"));
}

#[test]
fn the_full_claude_code_document_maps_field_for_field() {
    let record = ingest(
        None,
        serde_json::json!({
            "hook_event_name": "Status",
            "session_id": "s-1",
            "transcript_path": "/tmp/t.jsonl",
            "cwd": "/tmp",
            "model": { "id": "claude-opus-4-1", "display_name": "Opus" },
            "workspace": { "current_dir": "/tmp", "project_dir": "/tmp" },
            "version": "1.0.80",
            "output_style": { "name": "default" },
            "cost": { "total_cost_usd": 0.01234, "total_duration_ms": 45000,
                      "total_lines_added": 156, "total_lines_removed": 23 },
            "context_window": {
                "total_input_tokens": 5000, "total_output_tokens": 400,
                "context_window_size": 200000,
                "current_usage": { "input_tokens": 1200, "output_tokens": 300,
                                   "cache_creation_input_tokens": 0,
                                   "cache_read_input_tokens": 900 }
            },
            "exceeds_200k_tokens": false
        }),
    )
    .expect("the documented document is accepted whole");
    assert_eq!(record.model_id.as_deref(), Some("claude-opus-4-1"));
    assert_eq!(record.total_cost_microdollars, Some(12_340));
    assert_eq!(record.context_window_size, Some(200_000));
    let usage = record.usage.expect("current usage present");
    assert_eq!(usage.input, Some(1200));
    assert_eq!(usage.output, Some(300));
    assert_eq!(usage.cache_creation_input, Some(0));
    assert_eq!(usage.cache_read_input, Some(900));
}

#[test]
fn a_session_alone_is_a_complete_record_with_nothing_measured() {
    let record = ingest(Some("s-1"), serde_json::json!({ "anything": "goes" }))
        .expect("unstored fields are ignored by name");
    assert_eq!(record.model_id, None);
    assert_eq!(record.total_cost_microdollars, None);
    assert_eq!(record.context_window_size, None);
    assert_eq!(record.usage, None);
}

#[test]
fn an_empty_model_id_is_absent_not_blank() {
    let record = ingest(Some("s-1"), serde_json::json!({ "model": { "id": "" } }))
        .expect("an empty id is not a rejection");
    assert_eq!(record.model_id, None);
}

#[test]
fn cost_out_of_range_is_rejected() {
    for bad in [
        f64::NAN,
        f64::INFINITY,
        -0.01,
        -0.000_000_1,
        i64::MAX as f64 / 1_000_000.0,
        1.0e300,
    ] {
        assert_eq!(
            usd_to_microdollars(bad),
            Err(StatusLineRejection::InvalidCost),
            "{bad}"
        );
    }
    assert_eq!(
        ingest(
            Some("s-1"),
            serde_json::json!({ "cost": { "total_cost_usd": -1.0 } })
        ),
        Err(StatusLineRejection::InvalidCost)
    );
}

#[test]
fn cost_rounds_to_the_nearest_microdollar() {
    assert_eq!(usd_to_microdollars(0.0), Ok(0));
    assert_eq!(usd_to_microdollars(0.42), Ok(420_000));
    assert_eq!(usd_to_microdollars(0.000_000_4), Ok(0));
    assert_eq!(usd_to_microdollars(0.000_000_6), Ok(1));
}

#[test]
fn a_negative_context_window_is_rejected() {
    assert_eq!(
        ingest(
            Some("s-1"),
            serde_json::json!({ "context_window": { "context_window_size": -1 } })
        ),
        Err(StatusLineRejection::NegativeContextWindow)
    );
}

#[test]
fn any_negative_token_count_is_rejected() {
    for field in [
        "input_tokens",
        "output_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
    ] {
        assert_eq!(
            ingest(
                Some("s-1"),
                serde_json::json!({ "context_window": { "current_usage": { field: -5 } } })
            ),
            Err(StatusLineRejection::NegativeTokenCount),
            "{field}"
        );
    }
}

#[test]
fn the_snapshot_carries_the_record_and_the_authenticated_user() {
    let record = ingest(
        Some("s-1"),
        serde_json::json!({ "model": { "id": "m" }, "cost": { "total_cost_usd": 1.5 } }),
    )
    .expect("accepted");
    let user = systemprompt::identifiers::UserId::new("u-1");
    let snapshot = record.snapshot(&user);
    assert_eq!(snapshot.session_id.as_str(), "s-1");
    assert_eq!(snapshot.user_id.as_str(), "u-1");
    assert_eq!(snapshot.model, Some("m"));
    assert_eq!(snapshot.total_cost_microdollars, Some(1_500_000));
    assert_eq!(snapshot.input_tokens, None);
}
