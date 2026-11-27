//! One open session as a GPUI entity: the pure model kept current from the
//! store and engine feeds, and the intents the transcript, docks and composer
//! call.
//!
//! Every engine or store call runs on tokio through `crate::app::runtime`; the
//! UI thread only applies results. Store changes and engine events arrive in
//! batches at most once a frame. While a load is in flight, changes are
//! held and applied after it, and streamed deltas are dropped (the model
//! reloads once the run is idle).

use std::path::PathBuf;

use gpui_kit::{Context, EventEmitter, Task};
use harness_engine::{
    CommandInput, EngineError, EngineEvent, PermissionId, PromptInput, QuestionId, Reply,
    SessionFailure,
};
use harness_store::{Change, MessageId, Session, SessionId};
use harness_workspace::FileDiff;

use crate::app::Services;
use crate::app::runtime::{self, FRAME};
use crate::state::input::{CommandCall, Selection, Submission};
use crate::state::loaders::load_session;
use crate::state::session_model::SessionModel;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SessionStateEvent {
    Forked(Box<Session>),
    Failed(SessionFailure),
}

#[derive(Debug, Default)]
pub(super) struct Held {
    pub(super) changes: Vec<Change>,
    pub(super) events: Vec<EngineEvent>,
}

pub(crate) struct SessionState {
    services: Services,
    pub(super) model: SessionModel,
    diffs: Vec<FileDiff>,
    pub(super) loading: bool,
    pub(super) held: Held,
    load_task: Option<Task<()>>,
    _feeds: [Task<()>; 2],
}

impl std::fmt::Debug for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionState")
            .field("id", self.model.id())
            .field("loading", &self.loading)
            .finish_non_exhaustive()
    }
}

impl EventEmitter<SessionStateEvent> for SessionState {}

impl SessionState {
    pub(crate) fn new(id: SessionId, services: Services, cx: &mut Context<Self>) -> Self {
        let changes = runtime::pump_batched(
            cx,
            services.store().subscribe(),
            FRAME,
            Self::on_changes,
            Self::on_lag,
        );
        let events = runtime::pump_batched(
            cx,
            services.engine().subscribe(),
            FRAME,
            Self::on_events,
            Self::on_lag,
        );
        let mut state = Self {
            services,
            model: SessionModel::new(id),
            diffs: Vec::new(),
            loading: false,
            held: Held::default(),
            load_task: None,
            _feeds: [changes, events],
        };
        state.reload(cx);
        state
    }

    pub(crate) const fn id(&self) -> &SessionId {
        self.model.id()
    }

    pub(crate) const fn model(&self) -> &SessionModel {
        &self.model
    }

    pub(crate) fn diffs(&self) -> &[FileDiff] {
        &self.diffs
    }

    pub(crate) fn directory(&self) -> Option<PathBuf> {
        self.model
            .session
            .as_ref()
            .map(|session| session.directory.clone())
    }

    pub(crate) fn seed(&mut self, session: Session) {
        self.model.seed(session);
    }

    pub(crate) fn reload(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        let engine = self.services.engine().clone();
        let id = self.id().clone();
        self.load_task = Some(runtime::spawn(
            cx,
            self.services.runtime(),
            load_session(engine, id),
            Self::loaded,
        ));
        cx.notify();
    }

    pub(crate) fn send(
        &mut self,
        submission: Submission,
        selection: Selection,
        cx: &Context<Self>,
    ) {
        let Some(directory) = self.directory() else {
            return;
        };
        let input = PromptInput {
            session_id: self.id().clone(),
            directory,
            text: submission.text,
            attachments: submission.attachments,
            agent: selection.agent,
            model: selection.model,
            delivery: submission.delivery,
        };
        let engine = self.services.engine().clone();
        self.model.set_error(None);
        self.run(cx, async move { engine.prompt(input).await.map(drop) });
    }

    pub(crate) fn run_command(
        &self,
        command: CommandCall,
        selection: Selection,
        cx: &Context<Self>,
    ) {
        let Some(directory) = self.directory() else {
            return;
        };
        let input = CommandInput {
            session_id: self.id().clone(),
            directory,
            name: command.name,
            arguments: command.arguments,
            agent: selection.agent,
            model: selection.model,
            delivery: command.delivery,
        };
        let engine = self.services.engine().clone();
        self.run(cx, async move { engine.command(input).await.map(drop) });
    }

    pub(crate) fn abort(&mut self, cx: &mut Context<Self>) {
        if self.services.engine().abort(self.model.id()) {
            cx.notify();
        }
    }

    pub(crate) fn reply_permission(
        &mut self,
        ask_id: &PermissionId,
        reply: Reply,
        cx: &mut Context<Self>,
    ) {
        if !self.services.engine().reply_permission(ask_id, reply) {
            tracing::debug!(ask = %ask_id, "permission request was already settled");
        }
        if self.model.resolve_permission(ask_id) {
            cx.notify();
        }
    }

    pub(crate) fn answer_question(
        &mut self,
        ask_id: &QuestionId,
        answers: Option<Vec<Vec<String>>>,
        cx: &mut Context<Self>,
    ) {
        if !self.services.engine().answer_question(ask_id, answers) {
            tracing::debug!(ask = %ask_id, "question was already settled");
        }
        if self.model.resolve_question(ask_id) {
            cx.notify();
        }
    }

    pub(crate) fn revert(&self, message: MessageId, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let id = self.id().clone();
        self.run(
            cx,
            async move { engine.revert(&id, &message).await.map(drop) },
        );
    }

    pub(crate) fn unrevert(&mut self, cx: &mut Context<Self>) {
        let engine = self.services.engine().clone();
        let id = self.id().clone();
        self.run(cx, async move { engine.unrevert(&id).await.map(drop) });
    }

    pub(crate) fn fork(&self, before: Option<MessageId>, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let id = self.id().clone();
        let job = async move { engine.fork(&id, before.as_ref()).await };
        runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            |this, result, cx| match result {
                Ok(forked) => cx.emit(SessionStateEvent::Forked(Box::new(forked))),
                Err(error) => this.fail(&error, cx),
            },
        )
        .detach();
    }

    pub(crate) fn load_diffs(&mut self, cx: &mut Context<Self>) {
        let engine = self.services.engine().clone();
        let id = self.id().clone();
        let job = async move { engine.diff(&id).await };
        runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            |this, result, cx| match result {
                Ok(diffs) => {
                    this.diffs = diffs;
                    cx.notify();
                },
                Err(error) => this.fail(&error, cx),
            },
        )
        .detach();
    }

    pub(crate) fn dismiss_error(&mut self, cx: &mut Context<Self>) {
        self.model.set_error(None);
        cx.notify();
    }

    fn run<F>(&self, cx: &Context<Self>, job: F)
    where
        F: Future<Output = Result<(), EngineError>> + Send + 'static,
    {
        runtime::spawn(cx, self.services.runtime(), job, |this, result, cx| {
            if let Err(error) = result {
                this.fail(&error, cx);
            }
        })
        .detach();
    }

    pub(super) fn fail(&mut self, error: &EngineError, cx: &mut Context<Self>) {
        tracing::warn!(session = %self.model.id(), kind = ?error.kind(), error = %error, "session action failed");
        let failure = SessionFailure::from(error);
        self.model.set_error(Some(failure.clone()));
        cx.emit(SessionStateEvent::Failed(failure));
        cx.notify();
    }
}
