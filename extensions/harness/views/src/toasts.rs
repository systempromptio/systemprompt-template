//! The window's toast host: shows the app state's notices in the toolkit's
//! notification stack.

use gpui_kit::{App, Window};
use systemprompt_harness_shared::components::Toast;

use crate::state::Notice;

pub(crate) fn show(notice: &Notice, window: &mut Window, cx: &mut App) {
    Toast::new(notice.tone, notice.message.clone()).show(window, cx);
}
