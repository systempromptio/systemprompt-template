//! The Settings screen: a list of pages on the left, the chosen page on the
//! right.
//!
//! Contract (fixed by the shell):
//!
//! - `SettingsView::new(app: Entity<AppState>, services: Services, window: &mut
//!   Window, cx: &mut Context<SettingsView>) -> SettingsView`; the frame builds
//!   it each time the route becomes `Route::Settings { page }` and opens `page`
//!   when it names a registered page. A later `Route::Settings` with another
//!   page, such as from a link, switches to it.
//! - The shell's pages are `SettingsPage` values in the registry `pages()`.
//!   Another module contributes a page by exposing a `fn settings_page() ->
//!   SettingsPage` and being listed in `pages()`;
//!   `crate::views::governance::settings_page` is listed and stays listed. A
//!   page's `build` creates its view entity. The admitted extensions' settings
//!   panes are listed after Governance and follow the enablement.
//! - The page list is keyboard-navigable (Up, Down, Home, End). Text comes from
//!   `crate::strings::settings`.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod about;
mod agents;
mod general;
mod layout;
mod mcp;
mod pages;
pub mod permissions;
mod reveal;
pub mod shortcuts;

use std::rc::Rc;

use gpui_kit::{
    AnyView, App, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled as _, Subscription, Window, div, px,
};
use harness_ui::IconName;
use harness_ui::components::{
    Dismiss, EmptyState, LIST_CONTEXT, ListRow, SelectFirst, SelectLast, SelectNext, SelectPrevious,
};
use harness_ui::theme::{space, tokens};

use crate::app::Services;
use crate::state::extensions::Contributions;
use crate::state::{AppState, Route};
use crate::strings::foundation as text;
use crate::views::settings::pages::Page;

const NAV_WIDTH: f32 = 220.0;

pub(crate) type PageBuilder = fn(Entity<AppState>, Services, &mut Window, &mut App) -> AnyView;

#[derive(Clone, Copy)]
pub(crate) struct SettingsPage {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) icon: IconName,
    pub(crate) build: PageBuilder,
}

impl std::fmt::Debug for SettingsPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsPage")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

pub(crate) fn pages() -> Vec<SettingsPage> {
    vec![
        general::settings_page(),
        permissions::settings_page(),
        agents::settings_page(),
        mcp::settings_page(),
        crate::views::governance::settings_page(),
        shortcuts::settings_page(),
        about::settings_page(),
    ]
}

pub(crate) struct SettingsView {
    app: Entity<AppState>,
    services: Services,
    contributions: Rc<Contributions>,
    pages: Vec<Page>,
    selected: Option<usize>,
    content: Option<AnyView>,
    requested: Option<String>,
    focus: FocusHandle,
    _observation: Subscription,
}

impl std::fmt::Debug for SettingsView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsView")
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl SettingsView {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let contributions = Rc::clone(&app.read(cx).contributions);
        let pages = pages::pages(&contributions);
        let requested = requested_page(&app, cx);
        let wanted = requested
            .as_deref()
            .and_then(|page| pages.iter().position(|candidate| candidate.id() == page));
        let observation = cx.observe_in(&app, window, |this, app, window, cx| {
            this.follow_route(&app, window, cx);
        });
        let focus = cx.focus_handle();
        let mut view = Self {
            app,
            services,
            contributions,
            pages,
            selected: None,
            content: None,
            requested,
            focus,
            _observation: observation,
        };
        view.select(wanted.unwrap_or(0), window, cx);
        view.focus.focus(window, cx);
        view
    }

    fn follow_route(
        &mut self,
        app: &Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.follow_contributions(app, window, cx);
        let requested = requested_page(app, cx);
        if requested == self.requested {
            return;
        }
        self.requested.clone_from(&requested);
        let wanted = requested.as_deref().and_then(|page| {
            self.pages
                .iter()
                .position(|candidate| candidate.id() == page)
        });
        if let Some(index) = wanted.filter(|index| self.selected != Some(*index)) {
            self.select(index, window, cx);
        }
    }

    fn follow_contributions(
        &mut self,
        app: &Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let contributions = Rc::clone(&app.read(cx).contributions);
        if Rc::ptr_eq(&contributions, &self.contributions) {
            return;
        }
        let shown = self
            .selected
            .and_then(|index| self.pages.get(index))
            .map(Page::id);
        self.pages = pages::pages(&contributions);
        self.contributions = contributions;
        let kept = shown.and_then(|id| self.pages.iter().position(|page| page.id() == id));
        self.selected = kept;
        if kept.is_none() {
            self.content = None;
            self.select(0, window, cx);
        }
        cx.notify();
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(page) = self.pages.get(index).cloned() else {
            return;
        };
        if self.selected == Some(index) && self.content.is_some() {
            return;
        }
        self.selected = Some(index);
        self.content = Some(page.build(&self.app, &self.services, window, cx));
        cx.notify();
    }

    fn step(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.pages.len();
        if count == 0 {
            return;
        }
        let current = self.selected.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, count as isize - 1) as usize;
        self.select(next, window, cx);
    }
}

fn requested_page(app: &Entity<AppState>, cx: &App) -> Option<String> {
    match &app.read(cx).route {
        Route::Settings { page } => page.clone(),
        _ => None,
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let last = self.pages.len().saturating_sub(1);
        let rows = self.pages.iter().enumerate().map(|(index, page)| {
            ListRow::new(("settings-page", index), page.title())
                .icon(page.icon())
                .selected(self.selected == Some(index))
                .on_click(cx.listener(move |this, _, window, cx| this.select(index, window, cx)))
        });
        let content = self.content.as_ref().map_or_else(
            || {
                div()
                    .size_full()
                    .child(EmptyState::new(text::PLACEHOLDER_SETTINGS).icon(IconName::Settings))
            },
            |view| div().size_full().child(view.clone()),
        );
        let nav = div()
            .id("settings-pages")
            .key_context(LIST_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &SelectNext, window, cx| this.step(1, window, cx)))
            .on_action(
                cx.listener(|this, _: &SelectPrevious, window, cx| this.step(-1, window, cx)),
            )
            .on_action(cx.listener(|this, _: &SelectFirst, window, cx| this.select(0, window, cx)))
            .on_action(
                cx.listener(move |this, _: &SelectLast, window, cx| this.select(last, window, cx)),
            )
            .on_action(cx.listener(|this, _: &Dismiss, _, cx| {
                this.app
                    .update(cx, |app, cx| app.navigate(Route::Session, cx));
            }))
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(space::XXS)
            .w(px(NAV_WIDTH))
            .h_full()
            .p(space::MD)
            .border_r_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .children(rows);
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(nav)
            .child(div().flex_1().min_w_0().h_full().child(content))
    }
}
