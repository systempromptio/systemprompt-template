//! A centred placeholder for an empty list or view.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::icons::{Icon, IconName};
use crate::theme::{radius, size, space, text, tokens};

/// A centred placeholder for an empty list or view: icon, title, description
/// and actions.
#[derive(IntoElement)]
pub struct EmptyState {
    icon: Option<IconName>,
    title: SharedString,
    description: Option<SharedString>,
    actions: Vec<AnyElement>,
}

impl std::fmt::Debug for EmptyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmptyState")
            .field("title", &self.title)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

impl EmptyState {
    #[must_use]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            title: title.into(),
            description: None,
            actions: Vec::new(),
        }
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn action(mut self, element: impl IntoElement) -> Self {
        self.actions.push(element.into_any_element());
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = *tokens(cx);
        let has_actions = !self.actions.is_empty();
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(space::MD)
            .size_full()
            .p(space::XXL)
            .children(self.icon.map(|icon| {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(40.))
                    .rounded(radius::FULL)
                    .bg(p.surface.hover)
                    .child(Icon::new(icon).size(size::ICON_LG).color(p.text.secondary))
            }))
            .child(
                div()
                    .text_size(text::TITLE)
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .text_color(p.text.primary)
                    .text_center()
                    .child(self.title),
            )
            .children(self.description.map(|description| {
                div()
                    .max_w(px(420.))
                    .text_size(text::BODY)
                    .text_color(p.text.secondary)
                    .text_center()
                    .child(description)
            }))
            .children(has_actions.then(|| {
                div()
                    .flex()
                    .flex_row()
                    .gap(space::MD)
                    .pt(space::SM)
                    .children(self.actions)
            }))
    }
}
