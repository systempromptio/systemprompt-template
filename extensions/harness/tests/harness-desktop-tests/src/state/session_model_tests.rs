use systemprompt_engine::run::{EngineEvent, RunStatus};
use systemprompt_engine::store::{
    AssistantMessage, Change, MessageId, PartBody, PartId, SessionId, UserMessage,
};
use systemprompt_harness_views::state::session_model::ModelEffect;

use super::session_model_fixtures::{loaded, text_of, text_part};
use crate::TestResult;

#[test]
fn a_streamed_turn_builds_the_transcript_in_order() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let user = UserMessage::new(&crate::IDS, id.clone(), "build", "m");
    let assistant = AssistantMessage::new(&crate::IDS, id.clone(), user.id.clone(), "build", "m");
    let prompt = text_part(&id, &user.id, "run the tests");
    let reply = text_part(&id, &assistant.id, "");

    let script = [
        Change::MessageUpdated(user.clone().into()),
        Change::PartUpdated(prompt),
        Change::MessageUpdated(assistant.clone().into()),
        Change::PartUpdated(reply.clone()),
        Change::PartDelta {
            session_id: id.clone(),
            message_id: assistant.id.clone(),
            part_id: reply.id.clone(),
            delta: "All ".to_owned(),
        },
        Change::PartDelta {
            session_id: id,
            message_id: assistant.id,
            part_id: reply.id,
            delta: "green.".to_owned(),
        },
    ];
    for change in &script {
        assert_eq!(model.apply_change(change), ModelEffect::Changed);
    }
    assert_eq!(model.messages.len(), 2);
    assert_eq!(text_of(&model, 0, 0)?, "run the tests");
    assert_eq!(text_of(&model, 1, 0)?, "All green.");
    assert_eq!(
        model.last_user_message().map(|entry| entry.message.id()),
        Some(&user.id)
    );
    Ok(())
}

#[test]
fn changes_for_other_sessions_are_ignored() {
    let id = SessionId::mint(&crate::IDS);
    let other = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let message = UserMessage::new(&crate::IDS, other.clone(), "build", "m");
    let revision = model.revision;
    assert_eq!(
        model.apply_change(&Change::MessageUpdated(message.into())),
        ModelEffect::Unchanged
    );
    assert_eq!(
        model.apply_event(&EngineEvent::Status {
            session: other,
            status: RunStatus::Busy
        }),
        ModelEffect::Unchanged
    );
    assert_eq!(model.revision, revision);
}

#[test]
fn a_part_for_an_unknown_message_asks_for_a_reload() {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let part = text_part(&id, &MessageId::mint(&crate::IDS), "orphan");
    assert_eq!(
        model.apply_change(&Change::PartUpdated(part)),
        ModelEffect::Reload
    );
}

#[test]
fn a_delta_for_an_unknown_part_marks_the_model_stale_until_idle() {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let user = UserMessage::new(&crate::IDS, id.clone(), "build", "m");
    model.apply_change(&Change::MessageUpdated(user.clone().into()));
    model.apply_event(&EngineEvent::Status {
        session: id.clone(),
        status: RunStatus::Busy,
    });
    let lost = Change::PartDelta {
        session_id: id.clone(),
        message_id: user.id,
        part_id: PartId::mint(&crate::IDS),
        delta: "x".to_owned(),
    };
    assert_eq!(model.apply_change(&lost), ModelEffect::Unchanged);
    assert!(model.stale);
    assert_eq!(
        model.apply_event(&EngineEvent::Status {
            session: id,
            status: RunStatus::Idle
        }),
        ModelEffect::Reload
    );
}

#[test]
fn an_older_part_update_does_not_overwrite_a_newer_one() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let user = UserMessage::new(&crate::IDS, id.clone(), "build", "m");
    model.apply_change(&Change::MessageUpdated(user.clone().into()));
    let mut newer = text_part(&id, &user.id, "abc");
    newer.updated_at = 20;
    let mut older = newer.clone();
    older.updated_at = 10;
    if let PartBody::Text { text, .. } = &mut older.body {
        "ab".clone_into(text);
    }
    model.apply_change(&Change::PartUpdated(newer));
    assert_eq!(
        model.apply_change(&Change::PartUpdated(older)),
        ModelEffect::Unchanged
    );
    assert_eq!(text_of(&model, 0, 0)?, "abc");
    Ok(())
}

#[test]
fn removals_take_messages_and_parts_out() {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let user = UserMessage::new(&crate::IDS, id.clone(), "build", "m");
    let part = text_part(&id, &user.id, "hi");
    model.apply_change(&Change::MessageUpdated(user.clone().into()));
    model.apply_change(&Change::PartUpdated(part.clone()));
    let removed = Change::PartRemoved {
        session_id: id.clone(),
        message_id: user.id.clone(),
        part_id: part.id,
    };
    assert_eq!(model.apply_change(&removed), ModelEffect::Changed);
    assert_eq!(
        model.messages[0].parts,
        Vec::<systemprompt_engine::store::Part>::new()
    );
    let gone = Change::MessageRemoved {
        session_id: id,
        message_id: user.id,
    };
    assert_eq!(model.apply_change(&gone), ModelEffect::Changed);
    assert_eq!(
        model.messages,
        Vec::<systemprompt_engine::store::MessageWithParts>::new()
    );
    assert_eq!(model.apply_change(&gone), ModelEffect::Unchanged);
}
