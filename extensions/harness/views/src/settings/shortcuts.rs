//! The Shortcuts settings page: the app's keyboard commands from the keymap and
//! the keys every list and picker answers to, shown as keycaps.

use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, Render, Window,
};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::KeyHint;

use crate::app::Services;
use crate::keymap;
use crate::settings::{SettingsPage, layout};
use crate::state::AppState;
use crate::strings::settings as text;

pub const APP_KEYS: [(&str, &str); 6] = [
    (text::KEY_NEW_SESSION, keymap::NEW_SESSION_KEYS),
    (text::KEY_FOCUS_COMPOSER, keymap::FOCUS_COMPOSER_KEYS),
    (text::KEY_TOGGLE_SIDEBAR, keymap::TOGGLE_SIDEBAR_KEYS),
    (text::KEY_OPEN_SETTINGS, keymap::OPEN_SETTINGS_KEYS),
    (text::KEY_STOP, keymap::STOP_RUN_KEYS),
    (text::KEY_PALETTE, keymap::COMMAND_PALETTE_KEYS),
];

const LIST_KEYS: [(&str, &str); 6] = [
    (text::KEY_NEXT, "down"),
    (text::KEY_PREVIOUS, "up"),
    (text::KEY_FIRST, "home"),
    (text::KEY_LAST, "end"),
    (text::KEY_ACTIVATE, "enter"),
    (text::KEY_DISMISS, "escape"),
];

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "shortcuts",
        title: text::SHORTCUTS,
        icon: IconName::Terminal,
        build,
    }
}

fn build(
    _app: Entity<AppState>,
    _services: Services,
    _window: &mut Window,
    cx: &mut App,
) -> AnyView {
    cx.new(|_| ShortcutsPage).into()
}

pub fn key_caps(binding: &str) -> Vec<String> {
    let mut caps = Vec::new();
    let mut rest = binding;
    while !rest.is_empty() {
        let (token, tail) = match rest.find('-') {
            Some(0) => rest.split_at(1),
            Some(at) => (&rest[..at], &rest[at + 1..]),
            None => (rest, ""),
        };
        caps.push(text::key_name(token).map_or_else(|| token.to_uppercase(), str::to_owned));
        rest = tail;
    }
    caps
}

struct ShortcutsPage;

fn rows(keys: &[(&'static str, &'static str)], cx: &App) -> Vec<AnyElement> {
    keys.iter()
        .map(|(title, binding)| layout::row(*title, None, KeyHint::new(key_caps(binding)), cx))
        .collect()
}

impl Render for ShortcutsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = vec![
            layout::card(text::SHORTCUTS_APP, rows(&APP_KEYS, cx)),
            layout::card(text::SHORTCUTS_LISTS, rows(&LIST_KEYS, cx)),
        ];
        layout::page(
            "settings-shortcuts",
            text::SHORTCUTS,
            text::SHORTCUTS_BODY,
            body,
            cx,
        )
    }
}
