//! The sidebar: new session, the project's sessions grouped by day with a
//! search and a menu per session (compact, delete), and a footer with the
//! admitted extensions' screens, Settings and the account. The session list is
//! virtualised: only visible rows are built.

use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;

use chrono::{DateTime, Local, NaiveDate};
use gpui_kit::{
    AnyElement, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div, px, uniform_list,
};
use harness_store::SessionId;
use harness_ui::IconName;
use harness_ui::components::{
    Avatar, Button, EmptyState, IconButton, ListRow, SectionHeader, Spinner, TextField,
};
use harness_ui::theme::{space, tokens};

use crate::app::Services;
use crate::state::{AppState, Route, SessionRow};
use crate::strings::foundation as text;
use crate::views::session_menu::session_menu;
use crate::views::{extension_entries, projects};

const WIDTH: f32 = 264.0;

pub(crate) struct SidebarView {
    app: Entity<AppState>,
    _services: Services,
    search: Entity<TextField>,
    expanded: HashSet<SessionId>,
    rows: Rc<Vec<SessionRow>>,
    _observation: Subscription,
}

impl std::fmt::Debug for SidebarView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarView")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

impl SidebarView {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let target = app.clone();
        let search = cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(text::SEARCH_SESSIONS)
                .on_change(move |query, _, cx| {
                    let query = query.to_string();
                    target.update(cx, |app, cx| app.set_search(query, cx));
                })
        });
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            _services: services,
            search,
            expanded: HashSet::new(),
            rows: Rc::default(),
            _observation: observation,
        }
    }

    fn toggle(&mut self, id: &SessionId, cx: &mut Context<Self>) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.clone());
        }
        cx.notify();
    }

    fn render_rows(&self, range: Range<usize>, cx: &Context<Self>) -> Vec<AnyElement> {
        let rows = Rc::clone(&self.rows);
        range
            .filter_map(|index| rows.get(index).map(|row| self.render_row(index, row, cx)))
            .collect()
    }

    fn render_row(&self, index: usize, row: &SessionRow, cx: &Context<Self>) -> AnyElement {
        let SessionRow::Session {
            id,
            depth,
            has_children,
            expanded,
        } = row
        else {
            let SessionRow::Header(bucket) = row else {
                return div().into_any_element();
            };
            return SectionHeader::new(text::day(*bucket)).into_any_element();
        };
        let state = self.app.read(cx);
        let title = state
            .sessions
            .get(id)
            .map(|session| session.title.trim().to_owned())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| text::UNTITLED_SESSION.to_owned());
        let selected = state.current.as_ref() == Some(id);
        let busy = state.is_busy(id);
        let menu = session_menu(
            &self.app,
            id,
            SharedString::from(title.clone()),
            busy,
            index,
        );
        let app = self.app.clone();
        let target = id.clone();
        let mut item = ListRow::new(("session-row", index), title)
            .selected(selected)
            .on_click(move |_, _, cx| {
                let target = target.clone();
                app.update(cx, |app, cx| app.select_session(target, cx));
            });
        if *depth > 0 {
            item = item.leading(div().w(px(12.)));
        }
        let mut trailing = div().flex().flex_row().items_center();
        if busy {
            trailing = trailing.child(Spinner::new().size(px(12.)));
        } else if *has_children {
            let (icon, tooltip) = if *expanded {
                (IconName::ChevronDown, text::HIDE_SUBTASKS)
            } else {
                (IconName::ChevronRight, text::SHOW_SUBTASKS)
            };
            let parent = id.clone();
            trailing = trailing.child(
                IconButton::new(("session-expand", index), icon, tooltip)
                    .small()
                    .on_click(cx.listener(move |this, _, _, cx| this.toggle(&parent, cx))),
            );
        }
        item.trailing(trailing.child(menu)).into_any_element()
    }

    fn render_list(&mut self, cx: &Context<Self>) -> AnyElement {
        let state = self.app.read(cx);
        let today = Local::now().date_naive();
        let rows = state
            .sessions
            .rows(&state.search, today, &self.expanded, local_day);
        let empty = if state.search.trim().is_empty() {
            text::NO_SESSIONS_TITLE
        } else {
            text::NO_MATCHES
        };
        let searching = !state.search.trim().is_empty();
        self.rows = Rc::new(rows);
        if self.rows.is_empty() {
            let mut placeholder = EmptyState::new(empty).icon(IconName::ListTodo);
            if !searching {
                placeholder = placeholder.description(text::NO_SESSIONS_BODY);
            }
            return div().flex_1().child(placeholder).into_any_element();
        }
        uniform_list(
            "sessions",
            self.rows.len(),
            cx.processor(|this, range: Range<usize>, _, cx| this.render_rows(range, cx)),
        )
        .flex_1()
        .min_h_0()
        .into_any_element()
    }

    fn render_footer(&self, cx: &Context<Self>) -> AnyElement {
        let state = self.app.read(cx);
        let account = state
            .accounts
            .active_account()
            .filter(|_| !state.needs_sign_in())
            .map_or_else(
                || text::SIGNED_OUT.to_owned(),
                |account| {
                    account
                        .user
                        .clone()
                        .unwrap_or_else(|| account.label.clone())
                },
            );
        let app = self.app.clone();
        div()
            .flex()
            .flex_col()
            .gap(space::XXS)
            .p(space::SM)
            .children(extension_entries::sidebar_rows(&self.app, cx))
            .child(
                ListRow::new("sidebar-settings", text::SETTINGS)
                    .icon(IconName::Settings)
                    .on_click(move |_, _, cx| {
                        app.update(cx, |app, cx| {
                            app.navigate(Route::Settings { page: None }, cx);
                        });
                    }),
            )
            .child(
                ListRow::new("sidebar-account", SharedString::from(account.clone()))
                    .leading(Avatar::new(account)),
            )
            .into_any_element()
    }
}

fn local_day(ms: i64) -> NaiveDate {
    DateTime::from_timestamp_millis(ms)
        .map(|time| time.with_timezone(&Local).date_naive())
        .unwrap_or_default()
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let app = self.app.clone();
        let header = div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .p(space::SM)
            .child(
                Button::new("sidebar-new-session")
                    .label(text::NEW_SESSION)
                    .icon(IconName::Plus)
                    .primary()
                    .full_width()
                    .on_click(move |_, _, cx| app.update(cx, AppState::new_session)),
            )
            .child(self.search.clone());
        let list = self.render_list(cx);
        let footer = self.render_footer(cx);
        let open_app = self.app.clone();
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(px(WIDTH))
            .h_full()
            .border_r_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .child(header)
            .child(
                SectionHeader::new(text::SESSIONS).trailing(
                    IconButton::new("sidebar-open-folder", IconName::Folder, text::OPEN_FOLDER)
                        .small()
                        .on_click(move |_, _, cx| projects::pick_folder(open_app.clone(), cx)),
                ),
            )
            .child(list)
            .child(footer)
    }
}
