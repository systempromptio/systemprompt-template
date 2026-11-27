//! The admitted extensions' marks under an assistant message, each drawn by
//! its extension from the message as stored, and the tasks that draw the
//! transcript again when an adornment's own state changed.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::rc::Rc;

use gpui_kit::{AnyElement, App, Context, Entity, Window};
use harness_shell::OnChange;
use harness_store::{Message, MessageWithParts};

use crate::state::AppState;

use crate::views::transcript::TranscriptView;

impl TranscriptView {
    pub(super) fn render_adornments(
        &self,
        entry: &MessageWithParts,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        if !matches!(entry.message, Message::Assistant(_)) {
            return Vec::new();
        }
        let contributions = Rc::clone(&self.app.read(cx).contributions);
        contributions
            .adornments
            .iter()
            .filter_map(|adornment| {
                adornment
                    .item
                    .adorn(entry, self.services.shell(), window, cx)
            })
            .collect()
    }

    pub(super) fn follow_adornments(&mut self, app: &Entity<AppState>, cx: &mut Context<Self>) {
        let contributions = Rc::clone(&app.read(cx).contributions);
        if self
            .adornments_from
            .as_ref()
            .is_some_and(|from| Rc::ptr_eq(from, &contributions))
        {
            return;
        }
        let view = cx.entity().downgrade();
        let on_change: OnChange = Rc::new(move |cx: &mut App| {
            if view.update(cx, |_, cx| cx.notify()).is_err() {
                tracing::trace!("an adornment changed after its transcript closed");
            }
        });
        self.adornment_follows = contributions
            .adornments
            .iter()
            .filter_map(|adornment| adornment.item.follow(Rc::clone(&on_change), cx))
            .collect();
        self.adornments_from = Some(contributions);
        cx.notify();
    }
}
