//! The new-session screen shown above the composer when no session is selected.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Subscription, Window,
    div,
};
use harness_ui::IconName;
use harness_ui::components::EmptyState;

use crate::app::Services;
use crate::state::AppState;
use crate::strings::foundation as text;

pub(crate) struct HomeView {
    app: Entity<AppState>,
    _services: Services,
    _observation: Subscription,
}

impl std::fmt::Debug for HomeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HomeView").finish_non_exhaustive()
    }
}

impl HomeView {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            _services: services,
            _observation: observation,
        }
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app.read(cx);
        let hint = state.project.as_ref().map_or_else(
            || text::NO_PROJECT.to_owned(),
            |project| format!("{} {}", text::HOME_HINT, text::home_project(&project.name)),
        );
        div().size_full().child(
            EmptyState::new(text::HOME_TITLE)
                .icon(IconName::Sparkle)
                .description(hint),
        )
    }
}
