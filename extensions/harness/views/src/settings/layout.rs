//! The shared shape of a settings page: a scrolling column with a title and
//! description, cards of rows, and a row with a label on the left and its
//! control on the right.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, App, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_ui::components::{Card, EmptyState, Spinner};
use harness_ui::theme::{space, text as type_scale, tokens};

const PAGE_WIDTH: f32 = 760.0;

pub(super) fn page(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    body: Vec<AnyElement>,
    cx: &App,
) -> AnyElement {
    let palette = *tokens(cx);
    let header = div()
        .flex()
        .flex_col()
        .gap(space::XS)
        .child(
            div()
                .text_size(type_scale::HEADING)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text.primary)
                .child(title),
        )
        .child(
            div()
                .text_size(type_scale::BODY)
                .text_color(palette.text.secondary)
                .child(description),
        );
    div()
        .id(id)
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
                .children(body),
        )
        .into_any_element()
}

pub(super) fn card(title: impl Into<SharedString>, rows: Vec<AnyElement>) -> AnyElement {
    Card::new()
        .title(title)
        .padding(space::LG)
        .children(rows)
        .into_any_element()
}

pub(super) fn row(
    title: impl Into<SharedString>,
    description: Option<SharedString>,
    control: impl IntoElement,
    cx: &App,
) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::LG)
        .w_full()
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(space::XXS)
                .child(
                    div()
                        .text_size(type_scale::BODY)
                        .text_color(palette.text.primary)
                        .child(title.into()),
                )
                .children(description.map(|description| {
                    div()
                        .text_size(type_scale::SMALL)
                        .text_color(palette.text.muted)
                        .child(description)
                })),
        )
        .child(div().flex_shrink_0().child(control))
        .into_any_element()
}

pub(super) fn note(message: impl Into<SharedString>, cx: &App) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .text_size(type_scale::SMALL)
        .text_color(palette.text.muted)
        .child(message.into())
        .into_any_element()
}

pub(super) fn mono(value: impl Into<SharedString>, cx: &App) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .min_w_0()
        .truncate()
        .font_family(harness_ui::theme::mono_font(cx))
        .text_size(type_scale::MONO)
        .text_color(palette.text.secondary)
        .child(value.into())
        .into_any_element()
}

pub(super) fn loading() -> AnyElement {
    div()
        .flex()
        .justify_center()
        .p(space::XL)
        .child(Spinner::new())
        .into_any_element()
}

pub(super) fn empty(title: &'static str) -> AnyElement {
    EmptyState::new(title).into_any_element()
}
