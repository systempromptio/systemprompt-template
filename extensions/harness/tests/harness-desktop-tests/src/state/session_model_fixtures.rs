use std::path::PathBuf;

use systemprompt_engine::run::{CallId, PermissionAsk, PermissionId, RunStatus};
use systemprompt_engine::store::{
    DiffSummary, MessageId, Part, PartBody, ProjectId, Session, SessionId,
};
use systemprompt_harness_views::state::session_model::{SessionModel, Snapshot};

use crate::TestResult;

pub(crate) fn session(id: &SessionId) -> Session {
    Session {
        id: id.clone(),
        project_id: ProjectId::mint(&crate::IDS),
        parent_id: None,
        directory: PathBuf::from("/work"),
        title: "Fix the build".to_owned(),
        agent: None,
        model: None,
        permission: None,
        revert: None,
        summary: DiffSummary::default(),
        metadata: systemprompt_engine::store::JsonMap::new(),
        created_at: 1,
        updated_at: 1,
        archived_at: None,
    }
}

pub(crate) fn loaded(id: &SessionId) -> SessionModel {
    let mut model = SessionModel::new(id.clone());
    model.load(Snapshot {
        session: Some(session(id)),
        messages: Vec::new(),
        todos: Vec::new(),
        permissions: Vec::new(),
        questions: Vec::new(),
        status: RunStatus::Idle,
    });
    model
}

pub(crate) fn text_part(session: &SessionId, message: &MessageId, text: &str) -> Part {
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

pub(crate) fn permission(session: &SessionId) -> TestResult<PermissionAsk> {
    Ok(PermissionAsk {
        id: PermissionId::mint(&crate::IDS),
        session: session.clone(),
        call_id: CallId::try_new("call")?,
        tool: "shell".to_owned(),
        permission: "shell".to_owned(),
        patterns: vec!["cargo test".to_owned()],
        always: Vec::new(),
        title: "cargo test".to_owned(),
        metadata: serde_json::Value::Null,
    })
}

pub(crate) fn text_of(model: &SessionModel, message: usize, part: usize) -> TestResult<String> {
    Ok(model.messages[message].parts[part]
        .body
        .text()
        .ok_or("a text part")?
        .to_owned())
}
