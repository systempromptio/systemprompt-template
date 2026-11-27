//! The root view: titlebar, sidebar and main area, or the sign-in gate over
//! everything below the titlebar while no instance is signed in and until the
//! gate lets go of the window. It routes the main area between the session (or
//! the home screen), Settings and the admitted extensions' screens (each built
//! from its builder when its route opens and dropped when the route leaves it
//! or the enablement withdraws it), owns one transcript and docks pair
//! per open session and routes their events, hosts the toasts, tells the sync
//! whether the window is in front and handles the shell's keyboard commands.

mod render;

use std::collections::HashMap;
use std::rc::Rc;

use gpui_kit::{
    AnyElement, AnyView, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement as _, Styled as _, Subscription, Window, div,
};
use harness_shell::ScreenKey;
use harness_store::SessionId;

use crate::app::Services;
use crate::state::{AppEvent, AppState, Route};
use crate::views::composer::ComposerView;
use crate::views::docks::DocksView;
use crate::views::home::HomeView;
use crate::views::settings::SettingsView;
use crate::views::sidebar::SidebarView;
use crate::views::signin::SignInGate;
use crate::views::titlebar::TitleBarView;
use crate::views::transcript::{TranscriptEvent, TranscriptView};
use crate::views::{toasts, window_state};

struct SessionPane {
    id: SessionId,
    transcript: Entity<TranscriptView>,
    docks: Entity<DocksView>,
    _events: Subscription,
}

#[derive(Default)]
struct Screens {
    settings: Option<Entity<SettingsView>>,
    extension: HashMap<ScreenKey, AnyView>,
}

pub(crate) struct Frame {
    app: Entity<AppState>,
    services: Services,
    titlebar: Entity<TitleBarView>,
    sidebar: Entity<SidebarView>,
    home: Entity<HomeView>,
    composer: Entity<ComposerView>,
    signin: Entity<SignInGate>,
    screens: Screens,
    panes: Vec<SessionPane>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("panes", &self.panes.len())
            .finish_non_exhaustive()
    }
}

impl Frame {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let titlebar = cx.new(|cx| TitleBarView::new(app.clone(), services.clone(), window, cx));
        let sidebar = cx.new(|cx| SidebarView::new(app.clone(), services.clone(), window, cx));
        let home = cx.new(|cx| HomeView::new(app.clone(), services.clone(), window, cx));
        let composer = cx.new(|cx| ComposerView::new(app.clone(), services.clone(), window, cx));
        let signin = cx.new(|cx| SignInGate::new(app.clone(), services.clone(), window, cx));
        let subscriptions = vec![
            cx.observe_in(&app, window, |this, _, window, cx| this.sync(window, cx)),
            cx.subscribe_in(&app, window, |this, _, event: &AppEvent, window, cx| {
                this.on_app_event(event, window, cx);
            }),
            cx.observe_window_bounds(window, |this, window, _| this.remember_bounds(window)),
            cx.observe_window_appearance(window, |_, _, cx| harness_ui::theme::refresh(cx)),
            cx.observe_window_activation(window, |this, window, _| this.follow_activation(window)),
            cx.observe(&signin, |_, _, cx| cx.notify()),
        ];
        let focus = cx.focus_handle();
        let mut frame = Self {
            app,
            services,
            titlebar,
            sidebar,
            home,
            composer,
            signin,
            screens: Screens::default(),
            panes: Vec::new(),
            focus,
            _subscriptions: subscriptions,
        };
        frame.sync(window, cx);
        frame.follow_activation(window);
        frame.focus.focus(window, cx);
        frame
    }

    fn follow_activation(&self, window: &Window) {
        self.services.sync().set_focused(window.is_window_active());
    }

    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.app.read(cx);
        let open: Vec<SessionId> = state.open_sessions().cloned().collect();
        let route = state.route.clone();
        self.panes.retain(|pane| open.contains(&pane.id));
        for id in open {
            if self.panes.iter().any(|pane| pane.id == id) {
                continue;
            }
            let Some(session) = self.app.read(cx).session(&id) else {
                continue;
            };
            let services = self.services.clone();
            let app = self.app.clone();
            let transcript = cx
                .new(|cx| TranscriptView::new(session.clone(), app, services.clone(), window, cx));
            let docks = cx.new(|cx| DocksView::new(session, services, window, cx));
            let events = cx.subscribe_in(
                &transcript,
                window,
                |this, _, event: &TranscriptEvent, window, cx| {
                    this.on_transcript_event(event, window, cx);
                },
            );
            self.panes.push(SessionPane {
                id,
                transcript,
                docks,
                _events: events,
            });
        }
        self.sync_screens(&route, window, cx);
        cx.notify();
    }

    fn sync_screens(&mut self, route: &Route, window: &mut Window, cx: &mut Context<Self>) {
        match route {
            Route::Settings { .. } if self.screens.settings.is_none() => {
                let (app, services) = (self.app.clone(), self.services.clone());
                self.screens.settings =
                    Some(cx.new(|cx| SettingsView::new(app, services, window, cx)));
            },
            Route::Screen(key) if !self.screens.extension.contains_key(key) => {
                let contributions = Rc::clone(&self.app.read(cx).contributions);
                if let Some(screen) = contributions.screen(key) {
                    let view = (screen.build)(self.services.shell(), window, cx);
                    self.screens.extension.insert(key.clone(), view);
                }
            },
            Route::Session => self.screens.settings = None,
            _ => {},
        }
        self.screens
            .extension
            .retain(|key, _| matches!(route, Route::Screen(open) if open == key));
    }

    fn on_app_event(&self, event: &AppEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            AppEvent::Notify(notice) => toasts::show(notice, window, cx),
            AppEvent::FocusComposer => self.focus_composer(window, cx),
            AppEvent::Draft(draft) => self.draft(draft.clone(), window, cx),
        }
    }

    fn draft(&self, draft: String, window: &mut Window, cx: &mut Context<Self>) {
        self.composer
            .update(cx, |composer, cx| composer.set_draft(draft, window, cx));
        self.focus_composer(window, cx);
    }

    fn on_transcript_event(
        &self,
        event: &TranscriptEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TranscriptEvent::OpenSession(id) => {
                let id = id.clone();
                self.app.update(cx, |app, cx| app.select_session(id, cx));
            },
            TranscriptEvent::RestoreDraft(body) => {
                let body = body.clone();
                self.composer
                    .update(cx, |composer, cx| composer.set_draft(body, window, cx));
                self.focus_composer(window, cx);
            },
        }
    }

    fn focus_composer(&self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.composer.read(cx).focus_handle(cx);
        handle.focus(window, cx);
    }

    fn remember_bounds(&self, window: &Window) {
        let geometry = window_state::geometry(window.window_bounds());
        self.services
            .settings()
            .update(|settings| settings.window = Some(geometry));
    }

    fn stop_run(&self, cx: &mut Context<Self>) {
        let stopped = self.app.update(cx, AppState::abort_current);
        if !stopped {
            self.app
                .update(cx, |app, cx| app.navigate(Route::Session, cx));
        }
    }

    fn render_main(&self, cx: &Context<Self>) -> AnyElement {
        let state = self.app.read(cx);
        match &state.route {
            Route::Settings { .. } => self
                .screens
                .settings
                .clone()
                .map_or_else(|| div().into_any_element(), IntoElement::into_any_element),
            Route::Screen(key) => self
                .screens
                .extension
                .get(key)
                .cloned()
                .map_or_else(|| div().into_any_element(), IntoElement::into_any_element),
            Route::Session => {
                let pane = state
                    .current
                    .as_ref()
                    .and_then(|id| self.panes.iter().find(|pane| &pane.id == id));
                let column = div().flex().flex_col().size_full();
                match pane {
                    Some(pane) => column
                        .child(div().flex_1().min_h_0().child(pane.transcript.clone()))
                        .child(pane.docks.clone())
                        .child(self.composer.clone())
                        .into_any_element(),
                    None => column
                        .child(div().flex_1().min_h_0().child(self.home.clone()))
                        .child(self.composer.clone())
                        .into_any_element(),
                }
            },
        }
    }
}

impl Focusable for Frame {
    fn focus_handle(&self, _cx: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}
