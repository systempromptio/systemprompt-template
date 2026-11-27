//! The composer: a multi-line prompt that grows with its text, attachments, the
//! `/` command list and `@` file mentions, the inline agent and model pickers,
//! and send, steer, queue and stop.
//!
//! Contract (fixed by the shell): `ComposerView::new(app, services, window,
//! cx)` is built once for the window and mounted under the transcript or the
//! home screen. It is `Focusable` and takes a draft with `set_draft`. It
//! sends through `AppState::send` and stops through `AppState::abort_current`.
//! The unsent text and attachments are kept per session while the user moves
//! between sessions.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod attach;
pub mod drafts;
pub mod files;
pub mod input;
pub mod popup;
mod view;

use std::path::PathBuf;

use gpui_kit::component::input::TextareaState;
use gpui_kit::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, SharedString, Subscription,
    Task, Window,
};
use harness_store::SessionId;

use crate::app::{Services, runtime};
use crate::state::AppState;
use crate::strings::composer as text;
use crate::views::composer::drafts::{Attachment, Draft, Drafts};
use crate::views::composer::popup::Popup;
use crate::views::pickers::{AgentPicker, ModelPicker};

const MIN_ROWS: usize = 1;
const MAX_ROWS: usize = 10;

pub(crate) struct ComposerView {
    app: Entity<AppState>,
    services: Services,
    input: Entity<TextareaState>,
    model_picker: Entity<ModelPicker>,
    agent_picker: Entity<AgentPicker>,
    attachments: Vec<Attachment>,
    popup: Option<Popup>,
    files: Vec<String>,
    files_root: Option<PathBuf>,
    drafts: Drafts,
    session: Option<SessionId>,
    enter_sends: bool,
    busy_placeholder: bool,
    pastes: u32,
    walk: Option<Task<()>>,
    tasks: Vec<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for ComposerView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposerView")
            .field("attachments", &self.attachments.len())
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

impl ComposerView {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let enter_sends = services
            .settings()
            .read(|settings| settings.composer.enter_sends);
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(MIN_ROWS, MAX_ROWS)
                .submit_on_enter(enter_sends)
                .placeholder(text::PLACEHOLDER)
        });
        let model_picker = cx.new(|cx| ModelPicker::new(app.clone(), services.clone(), window, cx));
        let agent_picker = cx.new(|cx| AgentPicker::new(app.clone(), services.clone(), window, cx));
        let subscriptions = vec![
            cx.subscribe_in(&input, window, Self::on_input),
            cx.observe_in(&app, window, |this, _, window, cx| this.on_app(window, cx)),
        ];
        let session = app.read(cx).current.clone();
        let mut composer = Self {
            app,
            services,
            input,
            model_picker,
            agent_picker,
            attachments: Vec::new(),
            popup: None,
            files: Vec::new(),
            files_root: None,
            drafts: Drafts::default(),
            session,
            enter_sends,
            busy_placeholder: false,
            pastes: 0,
            walk: None,
            tasks: Vec::new(),
            _subscriptions: subscriptions,
        };
        composer.on_app(window, cx);
        composer
    }

    pub(crate) fn set_draft(&mut self, draft: String, window: &mut Window, cx: &mut Context<Self>) {
        self.popup = None;
        self.set_text(draft, window, cx);
        cx.notify();
    }

    fn text(&self, cx: &App) -> String {
        self.input.read(cx).value().to_string()
    }

    fn set_text(&self, value: String, window: &mut Window, cx: &mut Context<Self>) {
        let caret = value.len();
        self.input.update(cx, |input, cx| {
            input.set_value(SharedString::from(value), window, cx);
            input.set_selected_range(caret..caret, cx);
        });
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.retain(|task| !task.is_ready());
        self.tasks.push(task);
    }

    fn is_busy(&self, cx: &App) -> bool {
        let app = self.app.read(cx);
        app.current.as_ref().is_some_and(|id| app.is_busy(id))
    }

    fn has_model(&self, cx: &App) -> bool {
        !self.app.read(cx).models.is_empty()
    }

    fn on_app(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (current, directory) = {
            let app = self.app.read(cx);
            (
                app.current.clone(),
                app.project
                    .as_ref()
                    .map(|project| project.directory.clone()),
            )
        };
        if current != self.session {
            self.switch_session(current, window, cx);
        }
        if directory != self.files_root {
            self.load_files(directory, cx);
        }
        cx.notify();
    }

    fn switch_session(
        &mut self,
        next: Option<SessionId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let leaving = Draft {
            text: self.text(cx),
            attachments: std::mem::take(&mut self.attachments),
        };
        let previous = std::mem::replace(&mut self.session, next);
        self.drafts.keep(previous, leaving);
        let arriving = self.drafts.take(self.session.as_ref());
        self.attachments = arriving.attachments;
        self.popup = None;
        self.set_text(arriving.text, window, cx);
    }

    fn load_files(&mut self, directory: Option<PathBuf>, cx: &Context<Self>) {
        self.files.clear();
        self.files_root.clone_from(&directory);
        self.walk = directory.map(|root| {
            let walked = root.clone();
            runtime::blocking(
                cx,
                self.services.runtime(),
                move || files::walk(&walked, files::FILE_LIMIT),
                move |this, found, _| {
                    if this.files_root.as_ref() == Some(&root) {
                        this.files = found;
                    }
                },
            )
        });
    }
}

impl Focusable for ComposerView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}
