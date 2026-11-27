//! The menu on a session row in the sidebar: compact the session now, or
//! delete it after a confirmation.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{App, Entity, SharedString, Window};
use harness_store::SessionId;
use harness_ui::IconName;
use harness_ui::components::{ButtonKind, DialogAction, DialogSpec, Menu, MenuItem};

use crate::state::AppState;
use crate::strings::foundation as text;

pub(crate) fn session_menu(
    app: &Entity<AppState>,
    id: &SessionId,
    title: SharedString,
    busy: bool,
    index: usize,
) -> Menu {
    let (compact_app, compact_id) = (app.clone(), id.clone());
    let compact = MenuItem::new(text::COMPACT_SESSION, move |_, cx| {
        let id = compact_id.clone();
        compact_app.update(cx, |app, cx| app.compact_session(id, cx));
    })
    .disabled(busy);
    let (delete_app, delete_id) = (app.clone(), id.clone());
    let delete = MenuItem::new(text::DELETE_SESSION, move |window, cx| {
        confirm_delete(&delete_app, &delete_id, &title, window, cx);
    })
    .icon(IconName::Trash);
    Menu::icon(
        ("session-menu", index),
        IconName::Ellipsis,
        text::SESSION_ACTIONS,
        [compact.into(), delete.into()],
    )
    .align_end()
}

fn confirm_delete(
    app: &Entity<AppState>,
    id: &SessionId,
    title: &str,
    window: &mut Window,
    cx: &mut App,
) {
    let (app, id) = (app.clone(), id.clone());
    let delete = DialogAction::new(text::DELETE, move |_, cx| {
        let id = id.clone();
        app.update(cx, |app, cx| app.delete_session(id, cx));
        true
    })
    .kind(ButtonKind::Danger);
    DialogSpec::new(text::DELETE_SESSION_TITLE)
        .description(text::delete_session_body(title))
        .confirm(delete)
        .cancel(DialogAction::new(text::CANCEL, |_, _| true))
        .open(window, cx);
}
