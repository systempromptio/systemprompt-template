//! The sync panel's summary counts, its expandable details and the conflict
//! rows.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use super::DriveView;
use crate::strings::sync as text;
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_ui::IconName;
use harness_ui::components::{IconButton, ListRow, SectionHeader};
use harness_ui::theme::{text as type_scale, tokens};
use sp_drive::DriveStatus;
use sp_drive::sync::ConflictNotice;

const DETAILS_HEIGHT: f32 = 260.0;

impl DriveView {
    pub(super) fn sync_summary(status: &DriveStatus, cx: &Context<Self>) -> Vec<AnyElement> {
        let palette = tokens(cx);
        let line = |content: String| {
            div()
                .text_size(type_scale::CAPTION)
                .text_color(palette.text.muted)
                .truncate()
                .child(content)
                .into_any_element()
        };
        let mut lines = Vec::new();
        if let Some(root) = &status.root {
            lines.push(
                ListRow::new("drive-sync-root", root.display().to_string())
                    .icon(IconName::Folder)
                    .tooltip(text::OPEN_FOLDER)
                    .on_click(cx.listener(|this, _, _, cx| this.open_local(None, cx)))
                    .into_any_element(),
            );
        }
        lines.push(line(text::counts(
            status.counts.synced,
            status.counts.pending_uploads,
        )));
        if let Some(at) = status.last_synced {
            lines.push(line(text::synced_at(at, chrono::Local::now())));
        }
        if let Some(problem) = &status.problem {
            lines.push(
                div()
                    .text_size(type_scale::CAPTION)
                    .text_color(palette.danger.fg)
                    .child(text::problem(problem.kind))
                    .into_any_element(),
            );
        }
        lines
    }

    pub(super) fn sync_details_list(status: &DriveStatus, cx: &Context<Self>) -> AnyElement {
        let mut list = div()
            .id("drive-sync-details")
            .flex()
            .flex_col()
            .max_h(px(DETAILS_HEIGHT))
            .overflow_y_scroll();
        if !status.conflicts.is_empty() {
            list = list
                .child(SectionHeader::new(text::CONFLICTS).count(status.conflicts.len()))
                .children(
                    status
                        .conflicts
                        .iter()
                        .enumerate()
                        .map(|(index, conflict)| Self::conflict_row(index, conflict, cx)),
                );
        }
        if !status.sent_for_review.is_empty() {
            list = list
                .child(SectionHeader::new(text::IN_REVIEW).count(status.sent_for_review.len()))
                .children(
                    status
                        .sent_for_review
                        .iter()
                        .enumerate()
                        .map(|(index, sent)| {
                            ListRow::new(("drive-sync-review", index), sent.path.clone())
                                .icon(IconName::File)
                        }),
                );
        }
        if !status.notices.is_empty() {
            list = list
                .child(SectionHeader::new(text::PUT_BACK).count(status.notices.len()))
                .children(status.notices.iter().enumerate().map(|(index, notice)| {
                    ListRow::new(("drive-sync-notice", index), notice.path.clone())
                        .icon(IconName::Undo)
                }));
        }
        list = list.child(SectionHeader::new(text::RECENT));
        if status.recent.is_empty() {
            list = list.child(super::preview_parts::muted(text::NO_ACTIVITY, cx));
        }
        let now = chrono::Local::now();
        list.children(status.recent.iter().enumerate().map(|(index, item)| {
            ListRow::new(("drive-sync-activity", index), item.path.clone()).subtitle(format!(
                "{} · {}",
                text::activity(&item.kind),
                crate::strings::when(&item.at.to_rfc3339(), now)
            ))
        }))
        .into_any_element()
    }

    pub(super) fn conflict_row(
        index: usize,
        conflict: &ConflictNotice,
        cx: &Context<Self>,
    ) -> AnyElement {
        let sibling = conflict.sibling.clone();
        let path = conflict.path.clone();
        let show_copy = IconButton::new(
            ("drive-conflict-copy", index),
            IconName::Copy,
            text::SHOW_COPY,
        )
        .small()
        .on_click(cx.listener(move |this, _, _, cx| this.reveal(&sibling, cx)));
        let open = IconButton::new(
            ("drive-conflict-open", index),
            IconName::ExternalLink,
            text::OPEN_FILE,
        )
        .small()
        .disabled(conflict.unplaced)
        .on_click(cx.listener(move |this, _, _, cx| this.open_local(Some(&path), cx)));
        ListRow::new(("drive-conflict", index), conflict.path.clone())
            .icon(IconName::Alert)
            .subtitle(conflict.sibling.clone())
            .trailing(div().flex().flex_row().child(show_copy).child(open))
            .into_any_element()
    }
}
