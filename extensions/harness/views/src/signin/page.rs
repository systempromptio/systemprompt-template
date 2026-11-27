//! The frame every step of the sign-in gate sits in: the product and step line,
//! the step's title and lede, and a centred, scrollable column.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_ui::motion;
use harness_ui::theme::{self, space, tokens};

use super::SignInGate;
use super::flow::Step;
use crate::strings::signin as copy;

const CARD_WIDTH: f32 = 440.0;
const PAGE_PADDING: f32 = 56.0;

impl SignInGate {
    fn render_header(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let (step, title, body) = match self.flow.step {
            Step::Connect => (
                copy::STEP_CONNECT,
                copy::CONNECT_TITLE,
                Some(copy::CONNECT_BODY),
            ),
            Step::Choose => (
                copy::STEP_CHOOSE,
                copy::CHOOSE_TITLE,
                Some(copy::CHOOSE_BODY),
            ),
            Step::Finishing | Step::Closing | Step::Closed => {
                (copy::STEP_CHOOSE, copy::FINISHING_TITLE, None)
            },
        };
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(
                div()
                    .text_size(theme::text::CAPTION)
                    .text_color(palette.text.muted)
                    .child(copy::eyebrow(self.services.identity().display_name(), step)),
            )
            .child(
                div()
                    .text_size(theme::text::HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .children(body.map(|body| {
                div()
                    .text_size(theme::text::BODY)
                    .text_color(palette.text.secondary)
                    .child(body)
            }))
            .into_any_element()
    }

    pub(super) fn render_page(&self, cx: &Context<Self>) -> AnyElement {
        let body = match self.flow.step {
            Step::Connect => self.render_connect(cx),
            Step::Choose => self.render_choose(cx),
            Step::Finishing | Step::Closing | Step::Closed => self.render_finishing(cx),
        };
        let card = div()
            .flex()
            .flex_col()
            .gap(space::XL)
            .w_full()
            .max_w(px(CARD_WIDTH))
            .child(self.render_header(cx))
            .child(body);
        div()
            .id("signin-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .w_full()
                    .px(space::XXL)
                    .py(px(PAGE_PADDING))
                    .child(motion::fade_in("signin-card", card, cx)),
            )
            .into_any_element()
    }
}
