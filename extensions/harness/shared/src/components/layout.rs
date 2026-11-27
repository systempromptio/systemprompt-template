//! Structural pieces: section headers, dividers and cards.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::theme::{radius, size, space, text, tokens};

/// A small heading above a group of rows, with an optional count and trailing
/// action.
#[derive(IntoElement)]
pub struct SectionHeader {
    title: SharedString,
    count: Option<usize>,
    trailing: Option<AnyElement>,
}

impl std::fmt::Debug for SectionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SectionHeader")
            .field("title", &self.title)
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}

impl SectionHeader {
    #[must_use]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            count: None,
            trailing: None,
        }
    }

    #[must_use]
    pub const fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    #[must_use]
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = tokens(cx);
        let secondary = p.text.secondary;
        let muted = p.text.muted;
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .w_full()
            .h(size::CONTROL_SM)
            .px(space::MD)
            .text_size(text::CAPTION)
            .child(
                div()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .text_color(secondary)
                    .truncate()
                    .child(self.title),
            )
            .children(self.count.map(|count| {
                div()
                    .text_color(muted)
                    .child(SharedString::from(count.to_string()))
            }))
            .child(div().flex_1())
            .children(self.trailing)
    }
}

/// A one-pixel rule, horizontal or vertical, with an optional centred label.
#[derive(IntoElement, Debug, Clone)]
pub struct Divider {
    vertical: bool,
    label: Option<SharedString>,
}

impl Divider {
    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            vertical: false,
            label: None,
        }
    }

    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            vertical: true,
            label: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = tokens(cx);
        let line_color = p.border.divider;
        let muted = p.text.muted;
        if self.vertical {
            return div().flex_shrink_0().w(px(1.)).h_full().bg(line_color);
        }
        let line = || div().flex_1().h(px(1.)).bg(line_color);
        self.label.map_or_else(
            || div().w_full().h(px(1.)).flex_shrink_0().bg(line_color),
            |label| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(space::MD)
                    .w_full()
                    .text_size(text::CAPTION)
                    .text_color(muted)
                    .child(line())
                    .child(label)
                    .child(line())
            },
        )
    }
}

/// A raised container with a border, optional header row, and any children.
#[derive(IntoElement)]
pub struct Card {
    title: Option<SharedString>,
    trailing: Option<AnyElement>,
    padding: Pixels,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("title", &self.title)
            .field("children", &self.children.len())
            .finish_non_exhaustive()
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            title: None,
            trailing: None,
            padding: space::LG,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    #[must_use]
    pub const fn padding(mut self, padding: Pixels) -> Self {
        self.padding = padding;
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = *tokens(cx);
        let header = (self.title.is_some() || self.trailing.is_some()).then(|| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::MD)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(text::BODY_LARGE)
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .text_color(p.text.primary)
                        .children(self.title),
                )
                .children(self.trailing)
        });
        div()
            .flex()
            .flex_col()
            .gap(space::MD)
            .w_full()
            .p(self.padding)
            .rounded(radius::XL)
            .border_1()
            .border_color(p.border.subtle)
            .bg(p.surface.raised)
            .text_color(p.text.primary)
            .children(header)
            .children(self.children)
    }
}
