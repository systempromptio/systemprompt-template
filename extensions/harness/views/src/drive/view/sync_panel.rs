//! The local folder panel: where the sync stands, the folder with an Open
//! button, counts, the last problem, and on demand the conflicts, held deletes
//! with their two choices, edits sent for review and recent activity. Pause,
//! resume and Sync now act on the handle's sync; everything shown is read from
//! the sync's status feed when drawn.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::sync::Arc;

use gpui_kit::{
    AnyElement, App, Context, IntoElement, ParentElement as _, Styled as _, Window, div,
};
use harness_ui::components::{Banner, Button, IconButton, SectionHeader, StatusDot};
use harness_ui::theme::{space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};
use sp_drive::sync::HeldChoice;
use sp_drive::{DriveStatus, SyncPhase};

use super::DriveView;
use crate::bridge;
use crate::model::local_path;
use crate::strings::sync as text;

impl DriveView {
    fn toggle_pause(&self, cx: &mut Context<Self>) {
        let paused = self.sync_status.borrow().phase == SyncPhase::Paused;
        let sync = self.handle.sync();
        if paused {
            sync.resume();
        } else {
            sync.pause();
        }
        cx.notify();
    }

    fn choose_held(&mut self, choice: HeldChoice, cx: &Context<Self>) {
        let sync = Arc::clone(self.handle.sync());
        let job = async move { sync.choose_held_deletes(choice).await };
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-held-deletes",
            job,
            |this, result, cx| {
                if let Err(error) = result {
                    tracing::warn!(error = %error, "the choice for held deletes was not saved");
                    this.notify_user(Tone::Danger, text::CHOICE_FAILED, cx);
                }
            },
        );
        self.keep(task);
    }

    pub(super) fn reveal(&self, relative: &str, cx: &Context<Self>) {
        let path = {
            let status = self.sync_status.borrow();
            status
                .root
                .as_deref()
                .and_then(|root| local_path(root, relative))
        };
        if let Some(path) = path {
            cx.reveal_path(&path);
        }
    }

    pub(super) fn open_local(&self, relative: Option<&str>, cx: &Context<Self>) {
        let path = {
            let status = self.sync_status.borrow();
            status.root.as_deref().and_then(|root| {
                relative.map_or_else(
                    || Some(root.to_path_buf()),
                    |relative| local_path(root, relative),
                )
            })
        };
        if let Some(path) = path {
            cx.open_with_system(&path);
        }
    }

    fn sync_headline(status: &DriveStatus, cx: &Context<Self>) -> AnyElement {
        let tone = match status.phase {
            SyncPhase::Idle => Tone::Success,
            SyncPhase::Syncing => Tone::Accent,
            SyncPhase::Error => Tone::Danger,
            SyncPhase::Paused | SyncPhase::Disabled => Tone::Warning,
            SyncPhase::Off => Tone::Neutral,
        };
        let paused = status.phase == SyncPhase::Paused;
        let (pause_icon, pause_label) = if paused {
            (IconName::Send, text::RESUME)
        } else {
            (IconName::Stop, text::PAUSE)
        };
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .child(StatusDot::new("drive-sync-dot", tone).pulse(status.phase == SyncPhase::Syncing))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(type_scale::SMALL)
                    .child(text::phase(status.phase)),
            )
            .child(
                IconButton::new("drive-sync-pause", pause_icon, pause_label)
                    .small()
                    .disabled(status.phase == SyncPhase::Off)
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_pause(cx))),
            )
            .child(
                IconButton::new("drive-sync-now", IconName::Sync, text::SYNC_NOW)
                    .small()
                    .disabled(paused || status.phase == SyncPhase::Off)
                    .on_click(cx.listener(|this, _, _, _| this.handle.sync().sync_now())),
            )
            .into_any_element()
    }

    pub(super) fn render_sync(&self, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let (headline, summary, held, details) = {
            let status = self.sync_status.borrow();
            (
                Self::sync_headline(&status, cx),
                Self::sync_summary(&status, cx),
                status.held_deletes,
                self.sync_details
                    .then(|| Self::sync_details_list(&status, cx)),
            )
        };
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .p(space::SM)
            .border_t_1()
            .border_color(palette.border.subtle)
            .child(SectionHeader::new(text::TITLE))
            .child(headline)
            .children(summary);
        if let Some(held) = held {
            panel = panel
                .child(
                    Banner::warning("drive-sync-held", text::held(held.reason, held.count)).action(
                        text::RESTORE_DELETES,
                        Self::held_action(HeldChoice::Restore, cx),
                    ),
                )
                .child(
                    Button::new("drive-sync-apply")
                        .label(text::APPLY_DELETES)
                        .danger()
                        .small()
                        .on_click(
                            cx.listener(|this, _, _, cx| this.choose_held(HeldChoice::Apply, cx)),
                        ),
                );
        }
        let details_label = if self.sync_details {
            text::HIDE_DETAILS
        } else {
            text::DETAILS
        };
        panel = panel.child(
            Button::new("drive-sync-details")
                .label(details_label)
                .ghost()
                .small()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.sync_details = !this.sync_details;
                    cx.notify();
                })),
        );
        panel.children(details).into_any_element()
    }

    fn held_action(
        choice: HeldChoice,
        cx: &Context<Self>,
    ) -> impl Fn(&mut Window, &mut App) + 'static {
        let view = cx.weak_entity();
        move |_, cx| {
            if view
                .update(cx, |this, cx| this.choose_held(choice, cx))
                .is_err()
            {
                tracing::trace!("a held-delete choice came after the Drive closed");
            }
        }
    }
}
