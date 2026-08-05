//! The activity-feed constructors: the description text and metadata each
//! event lands in `user_activity` with.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin::activity::enums::entity_label;
use systemprompt_web_admin::activity::{
    ActivityAction, ActivityCategory, ActivityEntity, NewActivity,
};

fn user() -> UserId {
    UserId::new("user-1")
}

fn session() -> SessionId {
    SessionId::new("sess-1".to_owned())
}

#[test]
fn login_names_the_user_and_carries_no_metadata() {
    let activity = NewActivity::login(&user(), "Ada Lovelace");
    assert_eq!(activity.description, "Ada Lovelace logged in");
    assert!(matches!(activity.category, ActivityCategory::Login));
    assert!(matches!(activity.action, ActivityAction::LoggedIn));
    assert!(activity.entity.is_none());
    assert_eq!(activity.metadata, serde_json::json!({}));
}

#[test]
fn an_agent_response_quotes_its_preview_and_records_the_session() {
    let activity = NewActivity::agent_response(&user(), &session(), Some("all done"));
    assert_eq!(activity.description, "Claude responded: \"all done\"");
    assert_eq!(activity.metadata["session_id"], "sess-1");
}

#[test]
fn an_agent_response_without_a_preview_falls_back_to_a_fixed_line() {
    let activity = NewActivity::agent_response(&user(), &session(), None);
    assert_eq!(activity.description, "Claude finished responding");
}

#[test]
fn a_long_preview_is_truncated_at_a_word_boundary() {
    let message = "word ".repeat(40);
    let activity = NewActivity::agent_response(&user(), &session(), Some(&message));
    assert!(
        activity.description.ends_with("...\""),
        "{}",
        activity.description
    );
    assert!(activity.description.len() < message.len());
    assert!(!activity.description.contains("word...word"));
}

#[test]
fn a_preview_shorter_than_the_budget_is_left_alone() {
    let activity = NewActivity::agent_response(&user(), &session(), Some("short"));
    assert_eq!(activity.description, "Claude responded: \"short\"");
}

#[test]
fn truncation_does_not_split_a_multibyte_character() {
    // Why: a description is arbitrary user text; slicing at a byte budget used
    // to panic mid-character on anything non-ASCII.
    let message = "é".repeat(200);
    let activity = NewActivity::agent_response(&user(), &session(), Some(&message));
    assert!(activity.description.ends_with("...\""));

    let emoji = "🚀".repeat(200);
    let activity = NewActivity::notification(&user(), &session(), None, Some(&emoji));
    assert!(activity.description.ends_with("..."));
}

#[test]
fn a_notification_is_described_by_the_type_and_message_it_has() {
    let cases = [
        (
            Some("permission_prompt"),
            Some("Bash wants to run rm"),
            "Permission prompt: Bash wants to run rm",
        ),
        (Some("alert"), Some("disk full"), "alert: disk full"),
        (Some("alert"), None, "alert"),
        (None, Some("disk full"), "disk full"),
        (None, None, "Notification received"),
    ];
    for (ntype, message, expected) in cases {
        let activity = NewActivity::notification(&user(), &session(), ntype, message);
        assert_eq!(activity.description, expected);
        assert!(matches!(activity.category, ActivityCategory::Notification));
        assert_eq!(activity.metadata["session_id"], "sess-1");
    }
}

#[test]
fn entity_crud_descriptions_use_the_human_label_for_the_kind() {
    let created =
        NewActivity::entity_created(&user(), ActivityEntity::McpServer, "sf", "Salesforce");
    assert_eq!(created.description, "Created MCP server 'Salesforce'");
    assert!(matches!(created.action, ActivityAction::Created));
    assert!(matches!(
        created.category,
        ActivityCategory::MarketplaceEdit
    ));

    let updated = NewActivity::entity_updated(&user(), ActivityEntity::Skill, "s1", "Web Search");
    assert_eq!(updated.description, "Updated skill 'Web Search'");

    // Why: a delete drops the name deliberately — the entity is gone, and the
    // reference lives in the entity ref rather than the sentence.
    let deleted = NewActivity::entity_deleted(&user(), ActivityEntity::Plugin, "p1", "Demo");
    assert_eq!(deleted.description, "Deleted a plugin");

    let entity = deleted.entity.expect("entity ref recorded");
    assert_eq!(entity.id.as_deref(), Some("p1"));
    assert_eq!(entity.name.as_deref(), Some("Demo"));
}

#[test]
fn every_entity_kind_has_a_label() {
    let kinds = [
        (ActivityEntity::Plugin, "plugin"),
        (ActivityEntity::Hook, "hook"),
        (ActivityEntity::Agent, "agent"),
        (ActivityEntity::McpServer, "MCP server"),
        (ActivityEntity::Skill, "skill"),
        (ActivityEntity::Marketplace, "marketplace"),
        (ActivityEntity::User, "user"),
        (ActivityEntity::Prompt, "prompt"),
        (ActivityEntity::Session, "session"),
        (ActivityEntity::Tool, "tool"),
        (ActivityEntity::GatewayRoute, "gateway route"),
    ];
    for (kind, label) in kinds {
        assert_eq!(entity_label(kind), label);
    }
}
