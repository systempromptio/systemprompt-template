//! The building blocks screens are made of. Each is a `RenderOnce` element with
//! a builder API, except [`TextField`], which owns editing state and is an
//! entity.
//!
//! No component carries user-facing text of its own: labels, tooltips and
//! messages are always passed in.

mod badge;
mod button;
mod dialog;
mod empty_state;
mod feedback;
mod icon_button;
mod indicators;
mod layout;
mod list_row;
mod menu;
mod select;
mod text_field;
mod toggle;

use std::rc::Rc;

use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{AnyView, App, ClickEvent, KeyBinding, Keystroke, SharedString, Window};

pub use badge::{Badge, StatusDot};
pub use button::{Button, ButtonKind, ButtonSize};
pub use dialog::{DialogAction, DialogSpec};
pub use empty_state::EmptyState;
pub use feedback::{Banner, Toast};
pub use icon_button::IconButton;
pub use indicators::{Avatar, KeyHint, ProgressBar, Spinner, initials};
pub use layout::{Card, Divider, SectionHeader};
pub use list_row::ListRow;
pub use menu::{Menu, MenuEntry, MenuItem};
pub use select::{Select, SelectOption};
pub use text_field::TextField;
pub use toggle::{Checkbox, Toggle};

/// A click callback shared by components.
pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
/// A callback with no event payload.
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App)>;

pub const LIST_CONTEXT: &str = "HarnessList";

gpui_kit::actions!(
    harness_ui,
    [
        SelectNext,
        SelectPrevious,
        SelectFirst,
        SelectLast,
        Activate,
        Dismiss,
    ]
);

pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some(LIST_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("home", SelectFirst, context),
        KeyBinding::new("end", SelectLast, context),
        KeyBinding::new("enter", Activate, context),
        KeyBinding::new("escape", Dismiss, context),
    ]);
}

pub fn tooltip(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |window, cx| Tooltip::new(text.clone()).build(window, cx)
}

pub fn tooltip_with_keys(
    text: impl Into<SharedString>,
    keystroke: &'static str,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    // Why: discard-ok: an unparsable shortcut is left out of the tooltip.
    let keys = Keystroke::parse(keystroke).ok();
    move |window, cx| {
        Tooltip::new(text.clone())
            .key_binding(keys.clone().map(Kbd::new))
            .build(window, cx)
    }
}
