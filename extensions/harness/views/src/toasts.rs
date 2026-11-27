//! The window's toast host: shows the app state's notices in the toolkit's
//! notification stack.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{App, Window};
use harness_ui::components::Toast;

use crate::state::Notice;

pub(crate) fn show(notice: &Notice, window: &mut Window, cx: &mut App) {
    Toast::new(notice.tone, notice.message.clone()).show(window, cx);
}
