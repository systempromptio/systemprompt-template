//! `user_activity` and `conversation_list` splice their inputs into SQL, so
//! what is pinned here is the alphabet: a quote, backslash or separator is
//! refused before any SQL is built, windows accept only the documented
//! shapes, and a name matches by id, email or display-name substring.

use serde_json::json;
use systemprompt_mcp_agent::typed::{
    ConversationListInput, UserActivityInput, conversation_list_command, user_activity_command,
    window_bound,
};

fn activity(value: serde_json::Value) -> UserActivityInput {
    serde_json::from_value(value).expect("valid input")
}

fn conversations(value: serde_json::Value) -> ConversationListInput {
    serde_json::from_value(value).expect("valid input")
}

#[test]
fn defaults_are_thirty_days_of_activity_and_seven_of_conversations() {
    let a = activity(json!({}));
    assert_eq!((a.since.as_str(), a.limit), ("30d", 20));
    let c = conversations(json!({}));
    assert_eq!((c.since.as_str(), c.limit, c.offset), ("7d", 25, 0));
}

#[test]
fn a_name_matches_id_email_and_display_name() {
    let command = user_activity_command(&activity(json!({"user": "Jay Dave"}))).expect("valid");
    assert!(command.starts_with("infra db query "));
    for part in [
        "u.id = ",
        "u.email = lower(",
        "COALESCE(u.display_name,",
        "%Jay Dave%",
    ] {
        assert!(command.contains(part), "missing {part}: {command}");
    }
    assert!(command.ends_with("--limit 20"), "{command}");
}

#[test]
fn quotes_and_separators_are_refused_before_sql_is_built() {
    for bad in ["x' OR '1'='1", "a;b", "a\\b", "a\"b", "a%b"] {
        assert!(
            user_activity_command(&activity(json!({"user": bad}))).is_err(),
            "{bad} must be refused"
        );
        assert!(conversation_list_command(&conversations(json!({"skill": bad}))).is_err());
    }
}

#[test]
fn windows_accept_relative_and_absolute_shapes_only() {
    assert_eq!(
        window_bound("since", "4w", "x").unwrap(),
        "now() - interval '4 weeks'"
    );
    assert_eq!(
        window_bound("since", "24h", "x").unwrap(),
        "now() - interval '24 hours'"
    );
    assert_eq!(window_bound("since", "", "dflt").unwrap(), "dflt");
    assert_eq!(
        window_bound("since", "2026-09-01", "x").unwrap(),
        "'2026-09-01'::timestamptz"
    );
    for bad in [
        "yesterday",
        "2026-9-1",
        "1y",
        "30dd",
        "2026-09-01'; --",
        "é",
    ] {
        assert!(
            window_bound("since", bad, "x").is_err(),
            "{bad} must be refused"
        );
    }
}

#[test]
fn conversation_list_pages_by_offset_and_filters_by_dashed_skill() {
    let command = conversation_list_command(&conversations(
        json!({"skill": "admin_ai_usage", "offset": 25, "limit": 10}),
    ))
    .expect("valid");
    assert!(command.contains("%admin-ai-usage%"));
    assert!(command.ends_with("--limit 10 --offset 25"));
}
