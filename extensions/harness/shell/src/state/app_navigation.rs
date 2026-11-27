//! Where the app is showing: the route, the sidebar, the session search and the
//! requests and links a launch hands over.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use crate::app::deeplink::{self, LaunchRequest, Link};
use crate::state::app_state::{AppEvent, AppState, Route};
use crate::strings::foundation as text;
use gpui_kit::Context;
use harness_shell::ScreenKey;
use harness_ui::Tone;

impl AppState {
    pub(crate) fn navigate(&mut self, route: Route, cx: &mut Context<Self>) {
        if self.route != route {
            self.route = route;
            cx.notify();
        }
    }

    pub(crate) fn toggle_screen(&mut self, key: ScreenKey, cx: &mut Context<Self>) {
        let route = Route::Screen(key);
        let next = if self.route == route {
            Route::Session
        } else {
            route
        };
        self.navigate(next, cx);
    }

    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_open = !self.sidebar_open;
        let open = self.sidebar_open;
        self.services
            .settings()
            .update(|settings| settings.sidebar_open = open);
        cx.notify();
    }

    pub(crate) fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        if self.search != query {
            self.search = query;
            cx.notify();
        }
    }

    pub(crate) fn handle_request(&mut self, request: &LaunchRequest, cx: &mut Context<Self>) {
        let scheme = self.services.identity().scheme();
        if let Some(directory) = request.project_directory(scheme) {
            self.open_project(directory, cx);
        }
        for link in &request.links {
            match deeplink::parse(scheme, link) {
                Ok(link) => self.apply_link(link, cx),
                Err(error) => {
                    tracing::warn!(error = %error, "link not understood");
                    Self::notify_user(Tone::Warning, text::LINK_NOT_UNDERSTOOD, cx);
                },
            }
        }
    }

    pub(super) fn apply_link(&mut self, link: Link, cx: &mut Context<Self>) {
        match link {
            Link::OpenProject { .. } => {},
            Link::NewSession { prompt, .. } => {
                self.new_session(cx);
                if let Some(prompt) = prompt {
                    cx.emit(AppEvent::Draft(prompt));
                }
            },
            Link::OpenSession { session } => self.select_session(session, cx),
            Link::OpenSettings { page } => self.navigate(Route::Settings { page }, cx),
        }
    }
}
