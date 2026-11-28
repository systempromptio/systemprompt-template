//! Sessions, messages and parts the tests build their transcripts from.

use std::path::PathBuf;

use serde_json::Value;
use systemprompt_engine::run::{CallId, RunStatus};
use systemprompt_engine::store::{
    AssistantMessage, DiffSummary, JsonMap, Message, MessageId, MessageWithParts, Part, PartBody,
    ProjectId, Session, SessionId, ToolState, UserMessage,
};
use systemprompt_harness_views::state::session_model::{SessionModel, Snapshot};

use crate::TestResult;

pub(crate) fn session(id: &SessionId, title: &str, updated_at: i64) -> Session {
    Session {
        id: id.clone(),
        project_id: ProjectId::mint(&crate::IDS),
        parent_id: None,
        directory: PathBuf::from("/work"),
        title: title.to_owned(),
        agent: None,
        model: None,
        permission: None,
        revert: None,
        summary: DiffSummary::default(),
        metadata: JsonMap::new(),
        created_at: updated_at,
        updated_at,
        archived_at: None,
    }
}

pub(crate) fn loaded(id: &SessionId) -> SessionModel {
    let mut model = SessionModel::new(id.clone());
    model.load(Snapshot {
        session: Some(session(id, "Fix the build", 1)),
        messages: Vec::new(),
        todos: Vec::new(),
        permissions: Vec::new(),
        questions: Vec::new(),
        status: RunStatus::Idle,
    });
    model
}

pub(crate) fn user(session: &SessionId) -> UserMessage {
    UserMessage::new(&crate::IDS, session.clone(), "build", "model")
}

pub(crate) fn assistant(session: &SessionId, parent: &MessageId) -> AssistantMessage {
    AssistantMessage::new(
        &crate::IDS,
        session.clone(),
        parent.clone(),
        "build",
        "model",
    )
}

pub(crate) fn text(session: &SessionId, message: &MessageId, text: &str) -> Part {
    Part::new(
        &crate::IDS,
        session.clone(),
        message.clone(),
        PartBody::Text {
            text: text.to_owned(),
            synthetic: false,
        },
    )
}

pub(crate) fn tool(
    session: &SessionId,
    message: &MessageId,
    name: &str,
    state: ToolState,
) -> TestResult<Part> {
    Ok(Part::new(
        &crate::IDS,
        session.clone(),
        message.clone(),
        PartBody::Tool {
            call_id: CallId::try_new("call-1")?,
            tool: name.to_owned(),
            state: Box::new(state),
        },
    ))
}

pub(crate) fn state(value: Value) -> TestResult<ToolState> {
    Ok(serde_json::from_value(value)?)
}

pub(crate) fn entry(message: impl Into<Message>, parts: Vec<Part>) -> MessageWithParts {
    MessageWithParts {
        message: message.into(),
        parts,
    }
}
