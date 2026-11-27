//! A dense, focusable list row: leading icon, title and subtitle, trailing
//! slot, selected and hover states.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};

use super::ClickHandler;
use crate::icons::{Icon, IconName};
use crate::theme::{Palette, radius, size, space, text, tokens};

/// A row in a sidebar, picker or settings list. With a click handler it is a
/// tab stop and Enter or Space activate it.
#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    icon: Option<IconName>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    selected: bool,
    disabled: bool,
    tall: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

impl std::fmt::Debug for ListRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListRow")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl ListRow {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            icon: None,
            leading: None,
            trailing: None,
            selected: false,
            disabled: false,
            tall: false,
            tooltip: None,
            on_click: None,
        }
    }

    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    #[must_use]
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    #[must_use]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn tall(mut self) -> Self {
        self.tall = true;
        self
    }

    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

fn leading_element(
    leading: Option<AnyElement>,
    icon: Option<IconName>,
    selected: bool,
    p: &Palette,
) -> Option<AnyElement> {
    match (leading, icon) {
        (Some(element), _) => Some(element),
        (None, Some(icon)) => Some(
            Icon::new(icon)
                .size(size::ICON_MD)
                .color(if selected {
                    p.text.primary
                } else {
                    p.text.secondary
                })
                .into_any_element(),
        ),
        (None, None) => None,
    }
}

fn text_column(
    title: SharedString,
    subtitle: Option<SharedString>,
    disabled: bool,
    p: &Palette,
) -> Div {
    let title_color = if disabled {
        p.text.disabled
    } else {
        p.text.primary
    };
    let detail_color = if disabled {
        p.text.disabled
    } else {
        p.text.muted
    };
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .child(div().truncate().text_color(title_color).child(title))
        .children(subtitle.map(|subtitle| {
            div()
                .truncate()
                .text_size(text::SMALL)
                .text_color(detail_color)
                .child(subtitle)
        }))
}

impl RenderOnce for ListRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = *tokens(cx);
        let interactive = self.on_click.is_some() && !self.disabled;
        let leading = leading_element(self.leading, self.icon, self.selected, &p);

        let mut row = div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .w_full()
            .min_h(if self.tall { size::ROW_TALL } else { size::ROW })
            .px(space::MD)
            .py(space::XXS)
            .rounded(radius::MD)
            .border_1()
            .border_color(p.surface.app.with_alpha(0.0))
            .text_size(text::BODY);
        if self.selected {
            row = row.bg(p.surface.selected);
        }
        if interactive {
            row = row
                .cursor_pointer()
                .tab_index(0)
                .hover(|this| {
                    this.bg(if self.selected {
                        p.surface.selected
                    } else {
                        p.surface.hover
                    })
                })
                .active(|this| this.bg(p.surface.active))
                .focus_visible(|this| this.border_color(p.border.focus));
        }
        if let Some(tooltip) = self.tooltip {
            row = row.tooltip(super::tooltip(tooltip));
        }
        if let (true, Some(handler)) = (interactive, self.on_click) {
            row = row.on_click(move |event, window, cx| handler(event, window, cx));
        }

        row.children(leading.map(|element| div().flex_shrink_0().child(element)))
            .child(text_column(self.title, self.subtitle, self.disabled, &p))
            .children(
                self.trailing
                    .map(|element| div().flex_shrink_0().ml(px(4.)).child(element)),
            )
    }
}
