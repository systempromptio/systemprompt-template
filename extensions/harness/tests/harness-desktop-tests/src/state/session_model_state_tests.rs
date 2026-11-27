use harness_desktop::state::session_model::{Effect, SessionModel, Snapshot};
use harness_engine::{
    CallId, EngineEvent, ErrorKind, QuestionAsk, QuestionId, Reply, RunStatus, SessionFailure,
};
use harness_store::{
    Change, DiffSummary, MessageWithParts, ProjectId, RevertMarker, SessionId, Todo, TodoPriority,
    TodoStatus, UserMessage,
};

use super::session_model_fixtures::{loaded, permission, session};

#[test]
fn status_permissions_and_questions_follow_the_engine() {
    let id = SessionId::mint();
    let mut model = loaded(&id);
    assert_eq!(
        model.apply_event(&EngineEvent::Status {
            session: id.clone(),
            status: RunStatus::Busy
        }),
        Effect::Changed
    );
    assert!(model.is_busy());
    let ask = permission(&id);
    let ask_id = ask.id.clone();
    assert_eq!(
        model.apply_event(&EngineEvent::PermissionAsked(Box::new(ask.clone()))),
        Effect::Changed
    );
    assert_eq!(
        model.apply_event(&EngineEvent::PermissionAsked(Box::new(ask))),
        Effect::Unchanged
    );
    let question_id = QuestionId::mint();
    let question = QuestionAsk {
        id: question_id.clone(),
        session: id.clone(),
        call_id: CallId::try_new("c").unwrap(),
        questions: Vec::new(),
    };
    model.apply_event(&EngineEvent::QuestionAsked(question));
    assert_eq!(model.permissions.len(), 1);
    assert_eq!(model.questions.len(), 1);
    let resolved = EngineEvent::PermissionResolved {
        id: ask_id,
        session: id.clone(),
        reply: Reply::Once,
    };
    assert_eq!(model.apply_event(&resolved), Effect::Changed);
    assert!(model.resolve_question(&question_id));
    assert!(!model.resolve_question(&question_id));
    assert!(model.permissions.is_empty() && model.questions.is_empty());
    assert_eq!(
        model.apply_event(&EngineEvent::Status {
            session: id,
            status: RunStatus::Idle
        }),
        Effect::Changed
    );
    assert!(!model.is_busy());
}

#[test]
fn a_session_error_shows_until_the_next_run() {
    let id = SessionId::mint();
    let mut model = loaded(&id);
    let error = SessionFailure {
        kind: ErrorKind::Model,
        message: "overloaded".to_owned(),
    };
    model.apply_event(&EngineEvent::SessionError {
        session: id.clone(),
        error: error.clone(),
    });
    assert_eq!(model.error, Some(error));
    model.apply_event(&EngineEvent::Status {
        session: id,
        status: RunStatus::Busy,
    });
    assert_eq!(model.error, None);
}

#[test]
fn todos_and_session_updates_replace_their_values() {
    let id = SessionId::mint();
    let mut model = loaded(&id);
    let todos = vec![Todo {
        content: "write tests".to_owned(),
        status: TodoStatus::InProgress,
        priority: TodoPriority::High,
    }];
    let change = Change::TodosUpdated {
        session_id: id.clone(),
        todos: todos.clone(),
    };
    assert_eq!(model.apply_change(&change), Effect::Changed);
    assert_eq!(model.apply_change(&change), Effect::Unchanged);
    assert_eq!(model.todos, todos);
    let mut renamed = session(&id);
    renamed.title = "Ship it".to_owned();
    renamed.summary = DiffSummary {
        additions: 3,
        deletions: 1,
        files: 2,
    };
    model.apply_change(&Change::SessionUpdated(Box::new(renamed)));
    assert_eq!(model.title(), Some("Ship it"));
    assert_eq!(model.diff().files, 2);
}

#[test]
fn a_revert_hides_messages_from_its_point_on() {
    let id = SessionId::mint();
    let mut model = loaded(&id);
    let first = UserMessage::new(id.clone(), "build", "m");
    let second = UserMessage::new(id.clone(), "build", "m");
    model.apply_change(&Change::MessageUpdated(first.into()));
    model.apply_change(&Change::MessageUpdated(second.clone().into()));
    let mut reverted = session(&id);
    reverted.revert = Some(RevertMarker {
        message_id: second.id,
        part_id: None,
        snapshot: None,
        diff: None,
    });
    model.apply_change(&Change::SessionUpdated(Box::new(reverted)));
    assert!(model.is_reverted());
    assert_eq!(model.visible_messages().len(), 1);
    assert_eq!(model.messages.len(), 2);
}

#[test]
fn deletion_clears_pending_work() {
    let id = SessionId::mint();
    let mut model = loaded(&id);
    model.apply_event(&EngineEvent::PermissionAsked(Box::new(permission(&id))));
    let deleted = Change::SessionDeleted {
        id,
        project_id: ProjectId::mint(),
        parent_id: None,
    };
    assert_eq!(model.apply_change(&deleted), Effect::Changed);
    assert!(model.deleted);
    assert_eq!(
        model.permissions,
        Vec::<harness_engine::PermissionAsk>::new()
    );
}

#[test]
fn load_sorts_and_filters_what_it_is_given() {
    let id = SessionId::mint();
    let first = UserMessage::new(id.clone(), "build", "m");
    let second = UserMessage::new(id.clone(), "build", "m");
    let mut model = SessionModel::new(id.clone());
    model.load(Snapshot {
        session: Some(session(&id)),
        messages: vec![
            MessageWithParts {
                message: second.into(),
                parts: Vec::new(),
            },
            MessageWithParts {
                message: first.clone().into(),
                parts: Vec::new(),
            },
        ],
        todos: Vec::new(),
        permissions: vec![permission(&id), permission(&SessionId::mint())],
        questions: Vec::new(),
        status: RunStatus::Busy,
    });
    assert_eq!(model.messages[0].message.id(), &first.id);
    assert_eq!(model.permissions.len(), 1);
    assert!(model.loaded && model.is_busy());
}

#[test]
fn effects_merge_towards_reload() {
    assert_eq!(Effect::Unchanged.merge(Effect::Changed), Effect::Changed);
    assert_eq!(Effect::Changed.merge(Effect::Reload), Effect::Reload);
    assert_eq!(
        Effect::Unchanged.merge(Effect::Unchanged),
        Effect::Unchanged
    );
}
