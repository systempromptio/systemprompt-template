//! The status lines under the conversation: the retry countdown, the error
//! banner with its dismiss, and the boundary left by an undo with the way back.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::time::Instant;

use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _,
    div,
};
use harness_engine::RunStatus;
use harness_ui::components::{Banner, Button, Spinner};
use harness_ui::theme::{size, space, text as type_scale, tokens};
use harness_ui::{Icon, IconName};

use crate::strings::transcript as text;
use crate::views::transcript::TranscriptView;

#[derive(Debug, Default)]
pub(crate) struct RetryClock {
    attempt: Option<u32>,
    since: Option<Instant>,
}

impl RetryClock {
    pub(crate) fn observe(&mut self, status: &RunStatus) {
        match status {
            RunStatus::Retry { attempt, .. } if self.attempt != Some(*attempt) => {
                self.attempt = Some(*attempt);
                self.since = Some(Instant::now());
            },
            RunStatus::Retry { .. } => {},
            RunStatus::Idle | RunStatus::Busy => {
                self.attempt = None;
                self.since = None;
            },
        }
    }

    pub(crate) fn seconds_left(&self, wait_ms: u64) -> u64 {
        let elapsed = self.since.map_or(0, |since| {
            u64::try_from(since.elapsed().as_millis()).unwrap_or(u64::MAX)
        });
        wait_ms.saturating_sub(elapsed).div_ceil(1_000)
    }
}

impl TranscriptView {
    pub(super) fn render_footer(&self, cx: &Context<Self>) -> AnyElement {
        let (status, error, reverted) = {
            let model = self.session.read(cx).model();
            (
                model.status.clone(),
                model.error.clone(),
                model.is_reverted(),
            )
        };
        let mut column = div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .w_full()
            .px(space::XL)
            .pb(space::SM);
        if let RunStatus::Retry {
            attempt,
            message,
            next_ms,
        } = &status
        {
            let line = text::retry(*attempt, self.retry.seconds_left(*next_ms), message);
            column = column.child(status_line(line, cx));
        }
        if let Some(failure) = error {
            let session = self.session.clone();
            column = column.child(
                Banner::error("transcript-error", failure.message)
                    .title(text::failure_title(failure.kind))
                    .dismissible(text::DISMISS, move |_, cx| {
                        session
                            .update(cx, crate::state::session_state::SessionState::dismiss_error);
                    }),
            );
        }
        if reverted {
            column = column.child(self.render_reverted(cx));
        }
        column.into_any_element()
    }

    fn render_reverted(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let session = self.session.clone();
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .w_full()
            .px(space::LG)
            .py(space::SM)
            .border_t_1()
            .border_b_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .text_size(type_scale::SMALL)
            .text_color(palette.text.secondary)
            .child(
                Icon::new(IconName::Undo)
                    .size(size::ICON_SM)
                    .color(palette.text.muted),
            )
            .child(div().flex_1().min_w_0().child(text::REVERTED))
            .child(
                Button::new("transcript-unrevert")
                    .label(text::UNREVERT)
                    .small()
                    .on_click(move |_, _, cx| {
                        session.update(cx, crate::state::session_state::SessionState::unrevert);
                    }),
            )
            .into_any_element()
    }
}

fn status_line(line: String, cx: &Context<TranscriptView>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::SM)
        .text_size(type_scale::SMALL)
        .text_color(palette.warning.fg)
        .child(Spinner::new().size(size::ICON_SM))
        .child(
            div()
                .font_weight(FontWeight::MEDIUM)
                .truncate()
                .child(SharedString::from(line)),
        )
        .into_any_element()
}
