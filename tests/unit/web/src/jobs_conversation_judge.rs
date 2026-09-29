//! The pure half of the conversation judge: the schema the provider
//! enforces, the normalisation of what comes back, and the transcript budget.
//!
//! The I/O half (discovery, leases, the judge call) needs a database and a
//! provider and belongs to the integration tier; what belongs here is every
//! decision where a wrong answer is silent — an unbounded tag list, a
//! confidence past 1, a transcript that quietly drops the closing turns.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use chrono::{DateTime, TimeZone, Utc};
use systemprompt::identifiers::{AiRequestId, GatewayConversationId};
use systemprompt_web_admin::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow,
};
use systemprompt_web_admin::test_support::{
    ConversationView, TranscriptOptions, build_conversation,
};
use systemprompt_web_extension::jobs::internals::{
    Category, Classification, Outcome, TranscriptMeta, classification_schema, parse_classification,
    render_transcript,
};

#[test]
fn the_schema_enumerates_every_category_and_outcome_and_requires_every_field() {
    let schema = classification_schema();
    let categories = schema["properties"]["category"]["enum"]
        .as_array()
        .expect("category enum");
    assert_eq!(categories.len(), 7);
    assert!(categories.iter().any(|c| c == "business-analysis"));
    let outcomes = schema["properties"]["outcome"]["enum"]
        .as_array()
        .expect("outcome enum");
    assert_eq!(outcomes.len(), 4);
    let required = schema["required"].as_array().expect("required list");
    for field in [
        "title",
        "category",
        "summary",
        "tags",
        "outcome",
        "skills_observed",
        "confidence",
        "completion",
        "completion_rationale",
    ] {
        assert!(
            required.iter().any(|r| r == field),
            "{field} must be required"
        );
    }
    // Why: Gemini's responseSchema rejects `additionalProperties`; the schema
    // must stay inside the OpenAPI subset the provider accepts.
    assert!(schema.get("additionalProperties").is_none());
}

#[test]
fn a_verdict_is_normalised_into_the_table_bounds() {
    let raw = r#"{
        "title": "  Fix the failing build  ",
        "category": "development",
        "summary": "  Fixed a failing build.  ",
        "tags": ["Rust", "rust", " CI ", "", "ci", "a", "b", "c", "d", "e", "f"],
        "outcome": "achieved",
        "skills_observed": ["Code-Review", "code-review"],
        "confidence": 1.7,
        "completion": 140,
        "completion_rationale": "The build passed after the fix."
    }"#;
    let parsed = parse_classification(raw).expect("valid verdict");
    assert_eq!(parsed.category, Category::Development);
    assert_eq!(parsed.title, "Fix the failing build");
    // Why: the score is clamped to the table's 0–100 range.
    assert_eq!(parsed.completion, 100);
    assert_eq!(parsed.outcome, Outcome::Achieved);
    assert_eq!(parsed.summary, "Fixed a failing build.");
    // Why: lowercased, deduplicated, and capped at eight.
    assert_eq!(parsed.tags.len(), 8);
    assert_eq!(&parsed.tags[..2], ["rust", "ci"]);
    assert_eq!(parsed.skills_observed, ["code-review"]);
    assert!((parsed.confidence - 1.0).abs() < f32::EPSILON);
}

#[test]
fn an_unknown_category_is_a_parse_error_not_a_silent_other() {
    let raw = r#"{"category": "gardening", "summary": "x", "tags": [], "outcome": "partial",
                  "skills_observed": [], "confidence": 0.5}"#;
    assert!(parse_classification(raw).is_err());
    let unreadable = Classification::unreadable();
    assert_eq!(unreadable.category, Category::Other);
    assert_eq!(unreadable.outcome, Outcome::Unclear);
}

fn at(secs: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + secs, 0)
        .single()
        .expect("valid timestamp")
}

fn request(id: &str, secs: i64, message_count: i64) -> ContextRequestRow {
    ContextRequestRow {
        id: AiRequestId::new(id),
        trace_id: None,
        model: Some("model-x".to_owned()),
        status: "completed".to_owned(),
        latency_ms: Some(10),
        input_tokens: Some(1),
        cache_read_tokens: None,
        cache_creation_tokens: None,
        output_tokens: Some(2),
        cost_microdollars: 5,
        created_at: at(secs),
        effective_kind: "turn".to_owned(),
        gateway_conversation_id: Some(
            GatewayConversationId::try_new("ctx_00000000000000aa").expect("thread id"),
        ),
        max_tokens: Some(1000),
        message_count,
    }
}

// Why: one request whose history holds `turns` user/assistant pairs, which is
// how the gateway records a conversation — the latest request carries it all.
fn conversation(turns: usize, prompt_len: usize) -> ConversationView {
    let mut rows = Vec::new();
    for i in 0..turns {
        rows.push(("user", format!("prompt {i} {}", "x".repeat(prompt_len))));
        rows.push(("assistant", format!("answer {i}")));
    }
    let messages: Vec<ContextMessageRow> = rows
        .iter()
        .enumerate()
        .map(|(seq, (role, content))| ContextMessageRow {
            request_id: AiRequestId::new("r1"),
            role: (*role).to_owned(),
            sequence_number: seq as i32,
            content: content.clone(),
            tool_call_id: None,
            name: None,
            created_at: at(1),
        })
        .collect();
    let requests = vec![request("r1", 1, messages.len() as i64)];
    build_conversation(&messages, &[], &requests, TranscriptOptions::owner_facing())
}

fn meta() -> TranscriptMeta {
    TranscriptMeta {
        client_kind: "claude-code".to_owned(),
        model: Some("model-x".to_owned()),
        turn_count: 3,
        tool_call_count: 0,
        error_count: 0,
        duration_minutes: 12,
        hooked_skills: vec!["ba-story-drafting".to_owned()],
    }
}

#[test]
fn a_short_conversation_renders_every_turn_after_its_metadata() {
    let text = render_transcript(&meta(), &conversation(3, 10), 24_000);
    assert!(text.starts_with("CONVERSATION METADATA\n"));
    assert!(text.contains("skills reported by the harness: ba-story-drafting"));
    assert!(text.contains("USER: prompt 0"));
    assert!(text.contains("ASSISTANT: answer 2"));
    assert!(!text.contains("turns omitted"));
}

#[test]
fn an_oversized_conversation_keeps_its_opening_and_closing_turns() {
    // Why: 40 turns of ~2k chars each is far past a 5k-token (~20k char) budget.
    let text = render_transcript(&meta(), &conversation(40, 2_000), 5_000);
    assert!(
        text.len() <= 5_000 * 4 + 64,
        "budget overrun: {}",
        text.len()
    );
    assert!(text.contains("USER: prompt 0 "), "opening turn dropped");
    assert!(text.contains("USER: prompt 39 "), "closing turn dropped");
    assert!(text.contains("turns omitted"), "no elision marker");
}
