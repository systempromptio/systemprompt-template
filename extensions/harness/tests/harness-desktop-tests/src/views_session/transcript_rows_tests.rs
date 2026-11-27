use systemprompt_engine::store::{
    AssistantMessage, Message, MessageId, MessageWithParts, Part, PartBody, SessionId, UserMessage,
};
use systemprompt_harness_views::transcript::rows::{changed_range, plain_text, row_keys};

fn user(session: &SessionId, text: &str, system: bool) -> MessageWithParts {
    let mut message = UserMessage::new(session.clone(), "build", "model");
    message.system = system;
    let id = message.id.clone();
    let part = Part::new(
        session.clone(),
        id,
        PartBody::Text {
            text: text.to_owned(),
            synthetic: false,
        },
    );
    MessageWithParts {
        message: Message::User(message),
        parts: vec![part],
    }
}

fn assistant(session: &SessionId, parent: &MessageId, body: Option<PartBody>) -> MessageWithParts {
    let message = AssistantMessage::new(session.clone(), parent.clone(), "build", "model");
    let id = message.id.clone();
    let parts = body
        .into_iter()
        .map(|body| Part::new(session.clone(), id.clone(), body))
        .collect();
    MessageWithParts {
        message: Message::Assistant(message),
        parts,
    }
}

#[test]
fn system_prompts_and_empty_replies_are_hidden() {
    let session = SessionId::mint();
    let first = user(&session, "hello", false);
    let parent = first.message.id().clone();
    let messages = vec![
        first,
        assistant(
            &session,
            &parent,
            Some(PartBody::StepStart { snapshot: None }),
        ),
        user(&session, "continue", true),
    ];
    let keys = row_keys(&messages, false);
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].index, 0);
}

#[test]
fn the_live_reply_is_shown_while_busy() {
    let session = SessionId::mint();
    let first = user(&session, "hello", false);
    let parent = first.message.id().clone();
    let messages = vec![first, assistant(&session, &parent, None)];
    assert_eq!(row_keys(&messages, true).len(), 2);
    assert_eq!(row_keys(&messages, false).len(), 1);
}

#[test]
fn growing_text_changes_only_the_last_row() {
    let session = SessionId::mint();
    let first = user(&session, "hello", false);
    let parent = first.message.id().clone();
    let reply = assistant(
        &session,
        &parent,
        Some(PartBody::Text {
            text: "a".to_owned(),
            synthetic: false,
        }),
    );
    let mut messages = vec![first, reply];
    let before = row_keys(&messages, true);
    if let PartBody::Text { text, .. } = &mut messages[1].parts[0].body {
        text.push_str(" longer");
    }
    let after = row_keys(&messages, true);
    assert_eq!(changed_range(&before, &after), Some((1..2, 1)));
    assert_eq!(changed_range(&after, &after), None);
}

#[test]
fn appended_rows_splice_at_the_end() {
    let session = SessionId::mint();
    let messages = vec![user(&session, "one", false), user(&session, "two", false)];
    let all = row_keys(&messages, false);
    assert_eq!(changed_range(&all[..1], &all), Some((1..1, 1)));
    assert_eq!(changed_range(&all, &[]), Some((0..2, 0)));
}

#[test]
fn plain_text_skips_synthetic_parts() {
    let session = SessionId::mint();
    let mut entry = user(&session, "visible", false);
    let id = entry.message.id().clone();
    entry.parts.push(Part::new(
        session,
        id,
        PartBody::Text {
            text: "attached".to_owned(),
            synthetic: true,
        },
    ));
    assert_eq!(plain_text(&entry), "visible");
}
