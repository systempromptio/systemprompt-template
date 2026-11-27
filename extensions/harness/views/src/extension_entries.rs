//! The entry points of the admitted extensions' screens: in the titlebar the
//! item an extension draws itself for a screen, or else an icon button, each
//! opening its screen and closing it again; in the sidebar a row per screen.

use std::collections::HashMap;
use std::rc::Rc;

use gpui_kit::{AnyElement, AnyView, App, Entity, IntoElement, Window};
use systemprompt_harness_shared::components::{IconButton, ListRow};
use systemprompt_harness_shared::contract::{OpenScreen, Placement, ScreenKey, ShellContext};

use crate::state::extensions::Contributions;
use crate::state::{AppState, Route};

pub(crate) fn titlebar_items(
    app: &Entity<AppState>,
    contributions: &Contributions,
    ctx: &ShellContext,
    window: &mut Window,
    cx: &mut App,
) -> HashMap<ScreenKey, AnyView> {
    contributions
        .placed(Placement::Titlebar)
        .filter_map(|(key, _)| {
            let item = contributions.titlebar_item(&key)?;
            let (app, target) = (app.clone(), key.clone());
            let open: OpenScreen = Rc::new(move |_, cx| {
                let target = target.clone();
                app.update(cx, |app, cx| app.toggle_screen(target, cx));
            });
            Some((key, (item.build)(ctx, open, window, cx)))
        })
        .collect()
}

pub(crate) fn titlebar_entries(
    app: &Entity<AppState>,
    items: &HashMap<ScreenKey, AnyView>,
    cx: &App,
) -> Vec<AnyElement> {
    let state = app.read(cx);
    state
        .contributions
        .placed(Placement::Titlebar)
        .enumerate()
        .map(|(index, (key, screen))| {
            if let Some(item) = items.get(&key) {
                return item.clone().into_any_element();
            }
            let app = app.clone();
            IconButton::new(("titlebar-screen", index), screen.icon, screen.title)
                .small()
                .on_click(move |_, _, cx| {
                    let key = key.clone();
                    app.update(cx, |app, cx| app.toggle_screen(key, cx));
                })
                .into_any_element()
        })
        .collect()
}

pub(crate) fn sidebar_rows(app: &Entity<AppState>, cx: &App) -> Vec<AnyElement> {
    let state = app.read(cx);
    state
        .contributions
        .placed(Placement::Sidebar)
        .enumerate()
        .map(|(index, (key, screen))| {
            let route = Route::Screen(key);
            let selected = state.route == route;
            let app = app.clone();
            ListRow::new(("sidebar-screen", index), screen.title)
                .icon(screen.icon)
                .selected(selected)
                .on_click(move |_, _, cx| app.update(cx, |app, cx| app.navigate(route.clone(), cx)))
                .into_any_element()
        })
        .collect()
}
