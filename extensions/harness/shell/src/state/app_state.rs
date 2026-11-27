//! The app-wide UI model as a GPUI entity: projects, the open project's
//! sessions, the current session, models and agents, the accounts snapshot that
//! gates sign-in, and where the main area is.
//!
//! It is kept current from four feeds on the UI thread: the accounts watch,
//! the store's change feed (sessions and projects), the engine's events (run
//! status, model list changes) and the instance's extension enablement, which
//! decides the extension screens, settings panes and transcript adornments on
//! offer. Intents live in `app_intents` and `session_actions`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gpui_kit::{Context, Entity, EventEmitter, SharedString, Task};
use harness_engine::{CommandInfo, ModelSpec, RunStatus};
use harness_shell::ScreenKey;
use harness_store::{Project, ProjectId, Session, SessionId};
use harness_ui::Tone;
use sp_accounts::AccountsSnapshot;
use sp_extension::EnabledExtensions;

use crate::app::Services;
use crate::app::runtime::{self, FRAME};
use crate::state::extensions::Contributions;
use crate::state::loaders::AgentInfo;
use crate::state::session_list::SessionList;
use crate::state::session_state::SessionState;

pub(super) const OPEN_SESSION_LIMIT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Route {
    Session,
    Settings { page: Option<String> },
    Screen(ScreenKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectInfo {
    pub(crate) directory: PathBuf,
    pub(crate) name: String,
    pub(crate) id: Option<ProjectId>,
}

impl ProjectInfo {
    pub(crate) fn owns(&self, session: &Session) -> bool {
        self.id.as_ref().map_or_else(
            || session.directory == self.directory,
            |id| &session.project_id == id,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Notice {
    pub(crate) tone: Tone,
    pub(crate) message: SharedString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AppEvent {
    Notify(Notice),
    FocusComposer,
    Draft(String),
}

pub(crate) struct AppState {
    pub(super) services: Services,
    pub(crate) projects: Vec<Project>,
    pub(crate) project: Option<ProjectInfo>,
    pub(crate) sessions: SessionList,
    pub(crate) current: Option<SessionId>,
    pub(super) open: Vec<(SessionId, Entity<SessionState>)>,
    pub(crate) statuses: HashMap<SessionId, RunStatus>,
    pub(crate) models: Vec<ModelSpec>,
    pub(crate) model: Option<String>,
    pub(crate) agents: Vec<AgentInfo>,
    pub(crate) agent: Option<String>,
    pub(crate) commands: Vec<CommandInfo>,
    pub(crate) accounts: AccountsSnapshot,
    pub(crate) route: Route,
    pub(super) enabled: Option<EnabledExtensions>,
    pub(crate) contributions: Rc<Contributions>,
    pub(crate) sidebar_open: bool,
    pub(crate) search: String,
    pub(crate) loading_project: bool,
    pub(super) project_task: Option<Task<()>>,
    pub(super) side_tasks: Vec<Task<()>>,
    _feeds: [Task<()>; 4],
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("project", &self.project)
            .field("current", &self.current)
            .field("route", &self.route)
            .finish_non_exhaustive()
    }
}

impl EventEmitter<AppEvent> for AppState {}

impl AppState {
    pub(crate) fn new(services: Services, cx: &Context<Self>) -> Self {
        let accounts = runtime::pump(
            cx,
            services.accounts().subscribe(),
            Self::on_accounts,
            |_, _, _| {},
        );
        let changes = runtime::pump_batched(
            cx,
            services.store().subscribe(),
            FRAME,
            Self::on_changes,
            Self::on_lag,
        );
        let events = runtime::pump(
            cx,
            services.engine().subscribe(),
            Self::on_event,
            Self::on_lag,
        );
        let enablement = runtime::pump(
            cx,
            services.enablement().subscribe(),
            Self::on_enablement,
            |_, _, _| {},
        );
        let enabled = services.enablement().current();
        let contributions = Rc::new(Contributions::read(
            services.shell_extensions(),
            enabled.as_ref(),
            services.shell(),
        ));
        let sidebar_open = services.settings().read(|settings| settings.sidebar_open);
        let mut state = Self {
            accounts: services.accounts().snapshot(),
            services,
            projects: Vec::new(),
            project: None,
            sessions: SessionList::default(),
            current: None,
            open: Vec::new(),
            statuses: HashMap::new(),
            models: Vec::new(),
            model: None,
            agents: Vec::new(),
            agent: None,
            commands: Vec::new(),
            route: Route::Session,
            enabled,
            contributions,
            sidebar_open,
            search: String::new(),
            loading_project: false,
            project_task: None,
            side_tasks: Vec::new(),
            _feeds: [accounts, changes, events, enablement],
        };
        state.reload_models(cx);
        state.reload_projects(cx);
        state
    }

    pub(crate) const fn services(&self) -> &Services {
        &self.services
    }

    pub(crate) fn needs_sign_in(&self) -> bool {
        self.accounts.needs_sign_in()
    }

    pub(crate) fn session(&self, id: &SessionId) -> Option<Entity<SessionState>> {
        self.open
            .iter()
            .find(|(open, _)| open == id)
            .map(|(_, entity)| entity.clone())
    }

    pub(crate) fn current_session(&self) -> Option<Entity<SessionState>> {
        self.current.as_ref().and_then(|id| self.session(id))
    }

    pub(crate) fn open_sessions(&self) -> impl Iterator<Item = &SessionId> {
        self.open.iter().map(|(id, _)| id)
    }

    pub(crate) fn is_busy(&self, id: &SessionId) -> bool {
        self.statuses
            .get(id)
            .is_some_and(|status| !matches!(status, RunStatus::Idle))
    }

    pub(crate) fn current_model(&self) -> Option<&ModelSpec> {
        self.model.as_ref().map_or_else(
            || self.models.first(),
            |id| self.models.iter().find(|model| &model.id == id),
        )
    }

    pub(crate) fn notify_user(
        tone: Tone,
        message: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        cx.emit(AppEvent::Notify(Notice {
            tone,
            message: message.into(),
        }));
    }

    pub(crate) fn reload_models(&mut self, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            async move { engine.models() },
            |this, models, cx| {
                this.models = models;
                if this
                    .model
                    .as_ref()
                    .is_some_and(|id| !this.models.iter().any(|model| &model.id == id))
                {
                    this.model = None;
                }
                cx.notify();
            },
        );
        self.keep(task);
    }

    pub(crate) fn reload_projects(&mut self, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let job = crate::state::loaders::load_projects(engine);
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            |this, result, cx| match result {
                Ok(projects) => {
                    this.projects = projects;
                    cx.notify();
                },
                Err(error) => tracing::warn!(error = %error, "project list not loaded"),
            },
        );
        self.keep(task);
    }

    pub(super) fn keep(&mut self, task: Task<()>) {
        self.side_tasks.retain(|task| !task.is_ready());
        self.side_tasks.push(task);
    }
}
