//! What the session menu asks of the app state: compact a session now, or
//! delete it once the person confirmed.
//!
//! A deleted session leaves the list through the store's change feed; a
//! compaction shows up in the transcript as its marker and summary. Either
//! failure is reported as a notice.

use gpui_kit::Context;
use harness_engine::ErrorKind;
use harness_store::SessionId;
use harness_ui::Tone;

use crate::app::runtime;
use crate::state::app_state::AppState;
use crate::strings::foundation as text;

impl AppState {
    pub(crate) fn compact_session(&mut self, id: SessionId, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let session = id.clone();
        let job = async move { engine.compact(&session).await };
        let task =
            runtime::spawn(
                cx,
                self.services.runtime(),
                job,
                move |_, result, cx| match result {
                    Ok(()) => Self::notify_user(Tone::Success, text::SESSION_COMPACTED, cx),
                    Err(error) => {
                        tracing::warn!(session = %id, error = %error, "session not compacted");
                        Self::notify_user(Tone::Danger, text::failure(error.kind()), cx);
                    },
                },
            );
        self.keep(task);
    }

    pub(crate) fn delete_session(&mut self, id: SessionId, cx: &Context<Self>) {
        let engine = self.services.engine().clone();
        let session = id.clone();
        let job = async move { engine.delete(&session).await };
        let task =
            runtime::spawn(
                cx,
                self.services.runtime(),
                job,
                move |_, result, cx| match result {
                    Ok(true) => tracing::info!(session = %id, "session deleted"),
                    Ok(false) => {
                        tracing::debug!(session = %id, "the session to delete was already gone");
                        Self::notify_user(Tone::Warning, text::failure(ErrorKind::NotFound), cx);
                    },
                    Err(error) => {
                        tracing::warn!(session = %id, error = %error, "session not deleted");
                        Self::notify_user(Tone::Danger, text::failure(error.kind()), cx);
                    },
                },
            );
        self.keep(task);
    }
}
