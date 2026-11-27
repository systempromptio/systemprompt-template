//! What views ask the app state to do: open a project, start, select, fork and
//! delete sessions, send from the composer, choose the model and agent,
//! navigate, and follow links and later starts.

use std::path::{Path, PathBuf};

use gpui_kit::{AppContext as _, Context, Entity};
use harness_engine::EngineError;
use harness_store::{Session, SessionId};
use harness_ui::Tone;

use crate::app::paths::display_name;
use crate::app::runtime;
use crate::state::app_state::{AppEvent, AppState, OPEN_SESSION_LIMIT, ProjectInfo, Route};
use crate::state::input::{Intent, Selection, Submission, classify};
use crate::state::loaders::{ProjectLoad, load_project};
use crate::state::session_state::{SessionState, SessionStateEvent};
use crate::strings::foundation as text;

impl AppState {
    pub(crate) fn open_project(&mut self, directory: PathBuf, cx: &mut Context<Self>) {
        let same = self
            .project
            .as_ref()
            .is_some_and(|project| project.directory == directory);
        if !same {
            self.project = Some(ProjectInfo {
                name: display_name(&directory),
                directory: directory.clone(),
                id: None,
            });
            self.sessions.clear();
            self.current = None;
            self.search.clear();
            let prefs = self
                .services
                .settings()
                .read(|settings| settings.project(&directory));
            self.model = prefs.model;
            self.agent = prefs.agent;
        }
        self.services
            .settings()
            .update(|settings| settings.remember_project(&directory));
        self.route = Route::Session;
        self.load_project(directory, cx);
        cx.notify();
    }

    pub(super) fn load_project(&mut self, directory: PathBuf, cx: &Context<Self>) {
        self.loading_project = true;
        let job = load_project(self.services.engine().clone(), directory.clone());
        let task = runtime::spawn(cx, self.services.runtime(), job, move |this, result, cx| {
            this.project_loaded(&directory, result, cx);
        });
        self.project_task = Some(task);
    }

    fn project_loaded(
        &mut self,
        directory: &Path,
        result: Result<ProjectLoad, EngineError>,
        cx: &mut Context<Self>,
    ) {
        if self
            .project
            .as_ref()
            .is_none_or(|project| project.directory.as_path() != directory)
        {
            return;
        }
        self.loading_project = false;
        match result {
            Ok(load) => {
                if let (Some(project), Some(first)) = (self.project.as_mut(), load.roots.first()) {
                    project.id = Some(first.project_id.clone());
                }
                self.sessions.replace(load.roots, load.children);
                self.agents = load.agents;
                self.commands = load.commands;
                let last = self
                    .services
                    .settings()
                    .read(|settings| settings.project(directory).last_session);
                if self.current.is_none()
                    && let Some(id) = last.filter(|id| self.sessions.contains(id))
                {
                    self.select_session(id, cx);
                }
            },
            Err(error) => {
                tracing::warn!(directory = %directory.display(), error = %error, "project not loaded");
                Self::notify_user(Tone::Danger, text::PROJECT_LOAD_FAILED, cx);
            },
        }
        cx.notify();
    }

    pub(crate) fn new_session(&mut self, cx: &mut Context<Self>) {
        self.current = None;
        self.route = Route::Session;
        cx.emit(AppEvent::FocusComposer);
        cx.notify();
    }

    pub(crate) fn select_session(&mut self, id: SessionId, cx: &mut Context<Self>) {
        self.ensure_open(&id, cx);
        if let Some(directory) = self
            .project
            .as_ref()
            .map(|project| project.directory.clone())
        {
            let remembered = id.clone();
            self.services.settings().update(|settings| {
                settings.project_mut(&directory).last_session = Some(remembered);
            });
        }
        self.current = Some(id);
        self.route = Route::Session;
        cx.notify();
    }

    pub(crate) fn ensure_open(
        &mut self,
        id: &SessionId,
        cx: &mut Context<Self>,
    ) -> Entity<SessionState> {
        if let Some(position) = self.open.iter().position(|(open, _)| open == id) {
            let entry = self.open.remove(position);
            let entity = entry.1.clone();
            self.open.push(entry);
            return entity;
        }
        let services = self.services.clone();
        let session_id = id.clone();
        let entity = cx.new(|cx| SessionState::new(session_id, services, cx));
        let subscription = cx.subscribe(&entity, Self::on_session_event);
        subscription.detach();
        self.open.push((id.clone(), entity.clone()));
        while self.open.len() > OPEN_SESSION_LIMIT {
            let Some(index) = self
                .open
                .iter()
                .position(|(open, _)| self.current.as_ref() != Some(open))
            else {
                break;
            };
            self.open.remove(index);
        }
        entity
    }

    fn on_session_event(
        &mut self,
        _session: Entity<SessionState>,
        event: &SessionStateEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            SessionStateEvent::Forked(session) => {
                self.sessions.upsert(session.as_ref().clone());
                self.select_session(session.id.clone(), cx);
            },
            SessionStateEvent::Failed(failure) => {
                if failure.kind == harness_engine::ErrorKind::NotSignedIn {
                    self.reload_models(cx);
                }
            },
        }
    }

    pub(crate) fn send(&mut self, submission: Submission, cx: &mut Context<Self>) {
        if submission.is_empty() {
            return;
        }
        let selection = Selection {
            agent: self.agent.clone(),
            model: self.model.clone(),
        };
        match self.current_session() {
            Some(session) => {
                let intent = classify(submission, &self.commands);
                session.update(cx, |session, cx| match intent {
                    Intent::Prompt(submission) => session.send(submission, selection, cx),
                    Intent::Command(command) => session.run_command(command, selection, cx),
                });
            },
            None => self.start_session(submission, selection, cx),
        }
    }

    fn start_session(
        &mut self,
        submission: Submission,
        selection: Selection,
        cx: &mut Context<Self>,
    ) {
        let Some(directory) = self
            .project
            .as_ref()
            .map(|project| project.directory.clone())
        else {
            Self::notify_user(Tone::Warning, text::NO_PROJECT, cx);
            return;
        };
        let engine = self.services.engine().clone();
        let job = async move { engine.create(&directory).await };
        let task =
            runtime::spawn(
                cx,
                self.services.runtime(),
                job,
                move |this, result, cx| match result {
                    Ok(session) => this.started(session, submission, selection, cx),
                    Err(error) => {
                        tracing::warn!(error = %error, "session not created");
                        Self::notify_user(Tone::Danger, text::failure(error.kind()), cx);
                    },
                },
            );
        self.keep(task);
    }

    fn started(
        &mut self,
        session: Session,
        submission: Submission,
        selection: Selection,
        cx: &mut Context<Self>,
    ) {
        let id = session.id.clone();
        self.sessions.upsert(session.clone());
        self.select_session(id.clone(), cx);
        let entity = self.ensure_open(&id, cx);
        let intent = classify(submission, &self.commands);
        entity.update(cx, |state, cx| {
            state.seed(session);
            match intent {
                Intent::Prompt(submission) => state.send(submission, selection, cx),
                Intent::Command(command) => state.run_command(command, selection, cx),
            }
        });
    }

    pub(crate) fn abort_current(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(session) = self.current_session() else {
            return false;
        };
        let busy = session.read(cx).model().is_busy();
        if busy {
            session.update(cx, SessionState::abort);
        }
        busy
    }

    pub(crate) fn set_model(&mut self, model: Option<String>, cx: &mut Context<Self>) {
        self.model.clone_from(&model);
        self.remember(cx, move |prefs| prefs.model = model);
    }

    pub(crate) fn set_agent(&mut self, agent: Option<String>, cx: &mut Context<Self>) {
        self.agent.clone_from(&agent);
        self.remember(cx, move |prefs| prefs.agent = agent);
    }

    fn remember(
        &self,
        cx: &mut Context<Self>,
        edit: impl FnOnce(&mut crate::app::settings::ProjectPrefs),
    ) {
        if let Some(directory) = self
            .project
            .as_ref()
            .map(|project| project.directory.clone())
        {
            self.services
                .settings()
                .update(|settings| edit(settings.project_mut(&directory)));
        }
        cx.notify();
    }
}
