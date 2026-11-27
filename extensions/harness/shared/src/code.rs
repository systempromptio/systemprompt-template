//! A read-only code block: language label, copy button, optional line numbers,
//! and a scroll cap.
//!
//! Without line numbers the body is a fenced block in the toolkit's text view,
//! which highlights it with the theme's syntax palette. With line numbers the
//! lines are laid out as rows beside their gutter so the two always align; that
//! layout is not highlighted.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::sync::Arc;

use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::text::{TextView, TextViewStyle};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled as _,
    Window, div, px,
};

use crate::theme::{self, radius, space, text};

/// A read-only block of source code. `language` is a fence info string (`rust`,
/// `ts`) used for highlighting and as the header text unless `label` is set;
/// `copy_button(tooltip)` adds a copy button.
#[derive(IntoElement, Debug, Clone)]
pub struct CodeBlock {
    id: ElementId,
    code: SharedString,
    language: Option<SharedString>,
    label: Option<SharedString>,
    line_numbers: bool,
    max_height: Option<Pixels>,
    copy_tooltip: Option<SharedString>,
}

impl CodeBlock {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            language: None,
            label: None,
            line_numbers: false,
            max_height: None,
            copy_tooltip: None,
        }
    }

    #[must_use]
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub const fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    #[must_use]
    pub const fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = Some(height);
        self
    }

    #[must_use]
    pub fn copy_button(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.copy_tooltip = Some(tooltip.into());
        self
    }
}

#[must_use]
pub fn fenced(code: &str, language: Option<&str>) -> String {
    let mut longest = 0;
    let mut run = 0;
    for ch in code.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat(longest.max(2) + 1);
    let info = language
        .map(|lang| lang.split_whitespace().next().unwrap_or_default())
        .unwrap_or_default();
    let body = code.strip_suffix('\n').unwrap_or(code);
    format!("{fence}{info}\n{body}\n{fence}")
}

fn numbered_rows(code: &str, mono: SharedString, palette: &theme::Palette) -> AnyElement {
    let lines: Vec<&str> = code
        .strip_suffix('\n')
        .unwrap_or(code)
        .split('\n')
        .collect();
    let digits = lines.len().max(1).to_string().len() as f32;
    let gutter = px(digits.mul_add(7.5, 16.));
    div()
        .flex()
        .flex_col()
        .min_w_full()
        .py(space::SM)
        .font_family(mono)
        .text_size(text::MONO)
        .line_height(text::MONO_LINE)
        .children(lines.into_iter().enumerate().map(|(index, line)| {
            div()
                .flex()
                .flex_row()
                .whitespace_nowrap()
                .child(
                    div()
                        .flex_shrink_0()
                        .w(gutter)
                        .pr(space::MD)
                        .text_right()
                        .text_color(palette.text.muted)
                        .child(SharedString::from((index + 1).to_string())),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .pr(space::LG)
                        .text_color(palette.text.primary)
                        .child(SharedString::from(line.trim_end_matches('\r').to_owned())),
                )
        }))
        .into_any_element()
}

impl CodeBlock {
    fn child_id(&self, name: &'static str) -> ElementId {
        ElementId::NamedChild(Arc::new(self.id.clone()), SharedString::new_static(name))
    }

    fn body(&self, palette: &theme::Palette, mono: SharedString) -> AnyElement {
        if self.line_numbers {
            return numbered_rows(&self.code, mono, palette);
        }
        let chrome = StyleRefinement::default().bg(palette.surface.sunken);
        let source = fenced(&self.code, self.language.as_deref());
        div()
            .px(space::MD)
            .child(
                TextView::markdown(self.child_id("text"), SharedString::from(source))
                    .selectable(true)
                    .style(TextViewStyle::default().code_block(chrome)),
            )
            .into_any_element()
    }

    fn header(&self, palette: &theme::Palette) -> Option<gpui_kit::Div> {
        let header_text = self.label.clone().or_else(|| self.language.clone());
        if header_text.is_none() && self.copy_tooltip.is_none() {
            return None;
        }
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .px(space::MD)
            .py(space::XXS)
            .border_b_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .text_size(text::SMALL)
            .text_color(palette.text.secondary)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(header_text.unwrap_or_default()),
            );
        if let Some(tooltip) = self.copy_tooltip.clone() {
            header = header.child(
                Clipboard::new(self.child_id("copy"))
                    .tooltip(tooltip)
                    .value(self.code.clone()),
            );
        }
        Some(header)
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = *theme::tokens(cx);
        let mono = theme::mono_font(cx);
        let body = self.body(&palette, mono);
        let header = self.header(&palette);

        let mut scroller = div()
            .id(self.child_id("body"))
            .w_full()
            .overflow_x_scroll()
            .child(body);
        if let Some(height) = self.max_height {
            scroller = scroller.max_h(height).overflow_y_scroll();
        }

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .rounded(radius::LG)
            .border_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.sunken)
            .overflow_hidden()
            .children(header)
            .child(scroller)
    }
}
