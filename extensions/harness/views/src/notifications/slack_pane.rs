//! The Slack settings pane.
//!
//! What Slack is for this app (activity the instance relays as notifications;
//! no token, no sending), its status as the inbox shows it, and the
//! Slack-originated items, all read from the Slack handle on every change.
//! Without a handle the pane shows Slack withheld.

use chrono::Utc;
use gpui_kit::{
    AnyElement, App, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use harness_ui::components::{Card, ListRow, StatusDot};
use harness_ui::theme::{space, text as type_scale, tokens};
use sp_slack::{SlackHandle, SlackStatus};

use crate::model::slack_tone;
use crate::{bridge, strings as text};

const PAGE_WIDTH: f32 = 760.0;

pub(crate) struct SlackPane {
    handle: Option<SlackHandle>,
    _follow: Option<Task<()>>,
}

impl SlackPane {
    pub(crate) fn new(handle: Option<SlackHandle>, cx: &Context<Self>) -> Self {
        let follow = handle
            .as_ref()
            .map(|handle| bridge::follow(cx, handle.subscribe(), |_: &mut Self, cx| cx.notify()));
        Self {
            handle,
            _follow: follow,
        }
    }

    fn status(&self) -> SlackStatus {
        self.handle
            .as_ref()
            .map_or(SlackStatus::Withheld, SlackHandle::status)
    }

    fn render_items(&self, cx: &App) -> AnyElement {
        let now = Utc::now();
        let items = self
            .handle
            .as_ref()
            .map_or_else(Vec::new, SlackHandle::items);
        let mut card = Card::new().title(text::SLACK_ITEMS_TITLE);
        if items.is_empty() {
            card = card.child(paragraph(text::SLACK_NO_ITEMS, cx));
        }
        card.children(items.into_iter().enumerate().map(|(index, item)| {
            let mut row = ListRow::new(("slack-item", index), item.title).trailing(
                div()
                    .text_size(type_scale::CAPTION)
                    .text_color(tokens(cx).text.muted)
                    .child(text::relative(item.created_at, now)),
            );
            if !item.body.is_empty() {
                row = row.subtitle(item.body);
            }
            row
        }))
        .into_any_element()
    }
}

fn paragraph(message: &'static str, cx: &App) -> AnyElement {
    div()
        .text_size(type_scale::BODY)
        .text_color(tokens(cx).text.secondary)
        .child(message)
        .into_any_element()
}

fn status_card(status: SlackStatus, cx: &App) -> AnyElement {
    let palette = tokens(cx);
    Card::new()
        .title(text::SLACK_STATUS_TITLE)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .child(StatusDot::new("slack-status", slack_tone(status)))
                .child(
                    div()
                        .text_size(type_scale::BODY)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text.primary)
                        .child(text::slack_state(status)),
                ),
        )
        .child(
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(text::slack_detail(status, Utc::now())),
        )
        .into_any_element()
}

impl Render for SlackPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let status = self.status();
        let header = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(
                div()
                    .text_size(type_scale::HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(text::SLACK_TITLE),
            )
            .child(paragraph(text::SLACK_DESCRIPTION, cx));
        let how = Card::new()
            .title(text::SLACK_HOW_TITLE)
            .child(paragraph(text::SLACK_RELAYED, cx))
            .child(paragraph(text::SLACK_NO_TOKEN, cx));
        let items = (status != SlackStatus::Withheld).then(|| self.render_items(cx));
        div()
            .id("settings-slack")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XL)
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .p(space::XXL)
                    .child(header)
                    .child(how)
                    .child(status_card(status, cx))
                    .children(items),
            )
    }
}
