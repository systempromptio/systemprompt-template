//! Requests a shell extension's view makes of the frame around it, dispatched
//! as toolkit actions from the view's window and handled by the frame.

use gpui_kit::{Action, SharedString};

/// Opens the session and puts `text` into its composer as a draft for the user
/// to finish and send.
///
/// A view dispatches it with `Window::dispatch_action`; the frame handles it
/// on its root, so it reaches the frame from wherever focus is.
#[derive(Debug, Clone, PartialEq, Eq, Action)]
#[action(namespace = harness_shell, no_json)]
pub struct DraftInSession {
    pub text: SharedString,
}
