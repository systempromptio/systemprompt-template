//! Opening a folder in the system file manager from a settings page, off the UI
//! thread.

use std::path::PathBuf;

use gpui_kit::{Context, Entity, Task};
use harness_ui::Tone;

use crate::app::{Services, runtime};
use crate::state::AppState;
use crate::strings::settings as text;

pub(super) fn reveal<T: 'static>(
    folder: PathBuf,
    app: &Entity<AppState>,
    services: &Services,
    cx: &Context<T>,
) -> Task<()> {
    let app = app.clone();
    runtime::blocking(
        cx,
        services.runtime(),
        move || open::that_detached(&folder),
        move |_, result, cx| {
            if let Err(error) = result {
                tracing::warn!(error = %error, "folder not opened");
                app.update(cx, |_, cx| {
                    AppState::notify_user(Tone::Danger, text::OPEN_FAILED, cx);
                });
            }
        },
    )
}

pub(super) fn keep(tasks: &mut Vec<Task<()>>, task: Task<()>) {
    tasks.retain(|task| !task.is_ready());
    tasks.push(task);
}
