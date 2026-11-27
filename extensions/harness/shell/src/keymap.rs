//! The shell's keyboard commands and their bindings, registered once at
//! startup.
//!
//! The frame handles them in its key context; deeper contexts (lists, dialogs,
//! the composer) take precedence for the keys they bind.

use std::rc::Rc;

use gpui_kit::{Action, App, KeyBinding, KeyBindingContextPredicate};
use tracing::error;

pub(crate) const FRAME_CONTEXT: &str = "HarnessFrame";

gpui_kit::actions!(
    harness,
    [
        NewSession,
        FocusComposer,
        ToggleSidebar,
        OpenSettings,
        StopRun,
        CommandPalette,
    ]
);

pub(crate) const NEW_SESSION_KEYS: &str = "secondary-n";
pub(crate) const FOCUS_COMPOSER_KEYS: &str = "secondary-l";
pub(crate) const TOGGLE_SIDEBAR_KEYS: &str = "secondary-b";
pub(crate) const OPEN_SETTINGS_KEYS: &str = "secondary-,";
pub(crate) const STOP_RUN_KEYS: &str = "escape";
pub(crate) const COMMAND_PALETTE_KEYS: &str = "secondary-shift-p";

pub(crate) fn bind(cx: &mut App) {
    let commands: [(&str, Box<dyn Action>); 6] = [
        (NEW_SESSION_KEYS, Box::new(NewSession)),
        (FOCUS_COMPOSER_KEYS, Box::new(FocusComposer)),
        (TOGGLE_SIDEBAR_KEYS, Box::new(ToggleSidebar)),
        (OPEN_SETTINGS_KEYS, Box::new(OpenSettings)),
        (STOP_RUN_KEYS, Box::new(StopRun)),
        (COMMAND_PALETTE_KEYS, Box::new(CommandPalette)),
    ];
    let frame = Rc::new(KeyBindingContextPredicate::Identifier(FRAME_CONTEXT.into()));
    let bindings: Vec<KeyBinding> = commands
        .into_iter()
        .filter_map(|(keys, action)| binding(keys, action, &frame, cx))
        .collect();
    cx.bind_keys(bindings);
}

// Why: the toolkit resolves the `secondary` modifier (Cmd on macOS, Ctrl
// elsewhere) only in `KeyBinding::load`; `KeyBinding::new` rejects it and
// panics. A binding that does not parse leaves its command reachable from the
// palette and the menus, so it is reported and skipped.
fn binding(
    keys: &str,
    action: Box<dyn Action>,
    context: &Rc<KeyBindingContextPredicate>,
    cx: &App,
) -> Option<KeyBinding> {
    KeyBinding::load(keys, action, Some(Rc::clone(context)), false, None, cx)
        .inspect_err(|error| error!(keys, error = %error, "key binding rejected"))
        .ok()
}
