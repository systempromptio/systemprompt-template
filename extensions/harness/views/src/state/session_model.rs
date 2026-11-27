//! The pure model of one open session and the reducer that keeps it current.
//!
//! `SessionModel` is plain data: the session row, its messages with their parts
//! in creation order, the run status, the questions and permission requests
//! waiting for the user, the todo list, the changed-files summary and the error
//! to show. `SessionModel::apply_change` folds one store change into it and
//! `SessionModel::apply_event` one engine event; both say whether anything
//! visible changed, or that the model can no longer be trusted and must be
//! reloaded. No GPUI, no IO: the reducer is tested over scripted sequences.

use systemprompt_engine::run::{
    EngineEvent, PermissionAsk, PermissionId, QuestionAsk, QuestionId, RunStatus, SessionFailure,
};
use systemprompt_engine::store::{
    Change, DiffSummary, Message, MessageWithParts, Session, SessionId, Todo,
};

pub use crate::state::transcript::ModelEffect;
use crate::state::transcript::{self, remove_by};

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub session: Option<Session>,
    pub messages: Vec<MessageWithParts>,
    pub todos: Vec<Todo>,
    pub permissions: Vec<PermissionAsk>,
    pub questions: Vec<QuestionAsk>,
    pub status: RunStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionModel {
    id: SessionId,
    pub session: Option<Session>,
    pub messages: Vec<MessageWithParts>,
    pub status: RunStatus,
    pub permissions: Vec<PermissionAsk>,
    pub questions: Vec<QuestionAsk>,
    pub todos: Vec<Todo>,
    pub error: Option<SessionFailure>,
    pub loaded: bool,
    pub deleted: bool,
    pub stale: bool,
    pub revision: u64,
}

impl SessionModel {
    pub const fn new(id: SessionId) -> Self {
        Self {
            id,
            session: None,
            messages: Vec::new(),
            status: RunStatus::Idle,
            permissions: Vec::new(),
            questions: Vec::new(),
            todos: Vec::new(),
            error: None,
            loaded: false,
            deleted: false,
            stale: false,
            revision: 0,
        }
    }

    pub(crate) const fn id(&self) -> &SessionId {
        &self.id
    }

    pub const fn is_busy(&self) -> bool {
        !matches!(self.status, RunStatus::Idle)
    }

    pub fn title(&self) -> Option<&str> {
        self.session
            .as_ref()
            .map(|session| session.title.as_str())
            .filter(|title| !title.trim().is_empty())
    }

    pub fn diff(&self) -> DiffSummary {
        self.session
            .as_ref()
            .map(|session| session.summary)
            .unwrap_or_default()
    }

    pub fn visible_messages(&self) -> &[MessageWithParts] {
        let Some(marker) = self
            .session
            .as_ref()
            .and_then(|session| session.revert.as_ref())
        else {
            return &self.messages;
        };
        let end = self
            .messages
            .partition_point(|entry| entry.message.id() < &marker.message_id);
        &self.messages[..end]
    }

    pub fn is_reverted(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.revert.is_some())
    }

    pub fn last_user_message(&self) -> Option<&MessageWithParts> {
        self.visible_messages()
            .iter()
            .rev()
            .find(|entry| matches!(entry.message, Message::User(_)))
    }

    pub fn load(&mut self, snapshot: Snapshot) {
        let Snapshot {
            session,
            mut messages,
            todos,
            permissions,
            questions,
            status,
        } = snapshot;
        messages.sort_by(|a, b| a.message.id().cmp(b.message.id()));
        for entry in &mut messages {
            entry.parts.sort_by(|a, b| a.id.cmp(&b.id));
        }
        self.deleted = session.is_none();
        self.session = session;
        self.messages = messages;
        self.todos = todos;
        self.permissions = permissions
            .into_iter()
            .filter(|ask| ask.session == self.id)
            .collect();
        self.questions = questions
            .into_iter()
            .filter(|ask| ask.session == self.id)
            .collect();
        self.status = status;
        self.loaded = true;
        self.stale = false;
        self.bump();
    }

    pub fn apply_change(&mut self, change: &Change) -> ModelEffect {
        if change.session_id() != Some(&self.id) {
            return ModelEffect::Unchanged;
        }
        let effect = match change {
            Change::SessionCreated(session) | Change::SessionUpdated(session) => {
                self.session = Some(session.as_ref().clone());
                self.deleted = false;
                ModelEffect::Changed
            },
            Change::SessionDeleted { .. } => {
                self.deleted = true;
                self.status = RunStatus::Idle;
                self.permissions.clear();
                self.questions.clear();
                ModelEffect::Changed
            },
            Change::MessageUpdated(message) => {
                transcript::upsert_message(&mut self.messages, message)
            },
            Change::MessageRemoved { message_id, .. } => {
                transcript::remove_message(&mut self.messages, message_id)
            },
            Change::PartUpdated(part) => transcript::upsert_part(&mut self.messages, part),
            Change::PartRemoved {
                message_id,
                part_id,
                ..
            } => transcript::remove_part(&mut self.messages, message_id, part_id),
            Change::PartDelta {
                message_id,
                part_id,
                delta,
                ..
            } => transcript::append_delta(&mut self.messages, message_id, part_id, delta)
                .unwrap_or_else(|| {
                    self.stale = true;
                    ModelEffect::Unchanged
                }),
            Change::TodosUpdated { todos, .. } => {
                if &self.todos == todos {
                    ModelEffect::Unchanged
                } else {
                    self.todos.clone_from(todos);
                    ModelEffect::Changed
                }
            },
            Change::ProjectUpdated(_) | Change::ProjectDeleted { .. } => ModelEffect::Unchanged,
        };
        if effect == ModelEffect::Changed {
            self.bump();
        }
        effect
    }

    pub fn apply_event(&mut self, event: &EngineEvent) -> ModelEffect {
        let effect = match event {
            EngineEvent::Status { session, status } if session == &self.id => {
                if &self.status == status {
                    return ModelEffect::Unchanged;
                }
                let finished = matches!(status, RunStatus::Idle);
                self.status = status.clone();
                if matches!(status, RunStatus::Busy) {
                    self.error = None;
                }
                if finished && self.stale {
                    ModelEffect::Reload
                } else {
                    ModelEffect::Changed
                }
            },
            EngineEvent::PermissionAsked(ask) if ask.session == self.id => {
                if self.permissions.iter().any(|pending| pending.id == ask.id) {
                    return ModelEffect::Unchanged;
                }
                self.permissions.push(ask.as_ref().clone());
                ModelEffect::Changed
            },
            EngineEvent::PermissionResolved { id, session, .. } if session == &self.id => {
                remove_by(&mut self.permissions, |ask| &ask.id == id)
            },
            EngineEvent::QuestionAsked(ask) if ask.session == self.id => {
                if self.questions.iter().any(|pending| pending.id == ask.id) {
                    return ModelEffect::Unchanged;
                }
                self.questions.push(ask.clone());
                ModelEffect::Changed
            },
            EngineEvent::QuestionResolved { id, session, .. } if session == &self.id => {
                remove_by(&mut self.questions, |ask| &ask.id == id)
            },
            EngineEvent::SessionError { session, error } if session == &self.id => {
                self.error = Some(error.clone());
                ModelEffect::Changed
            },
            _ => ModelEffect::Unchanged,
        };
        if effect == ModelEffect::Changed {
            self.bump();
        }
        effect
    }

    pub(crate) fn resolve_permission(&mut self, id: &PermissionId) -> bool {
        let changed = remove_by(&mut self.permissions, |ask| &ask.id == id) == ModelEffect::Changed;
        if changed {
            self.bump();
        }
        changed
    }

    pub fn resolve_question(&mut self, id: &QuestionId) -> bool {
        let changed = remove_by(&mut self.questions, |ask| &ask.id == id) == ModelEffect::Changed;
        if changed {
            self.bump();
        }
        changed
    }

    pub(crate) fn set_error(&mut self, error: Option<SessionFailure>) {
        if self.error != error {
            self.error = error;
            self.bump();
        }
    }

    pub(crate) fn seed(&mut self, session: Session) {
        if self.session.is_none() {
            self.session = Some(session);
            self.bump();
        }
    }

    pub(crate) const fn mark_stale(&mut self) {
        self.stale = true;
    }

    const fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}
