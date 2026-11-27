//! Square, icon-only buttons whose tooltip doubles as their accessible name.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::rc::Rc;

use gpui_kit::component::{Disableable as _, Selectable as _};
use gpui_kit::{
    App, ClickEvent, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, Window,
};

use super::ClickHandler;
use super::button::{ButtonKind, ButtonSize, toolkit_button};
use crate::icons::IconName;

/// A square button showing only an icon. The tooltip doubles as its accessible
/// name, so it is required.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    tooltip: SharedString,
    kind: ButtonKind,
    size: ButtonSize,
    disabled: bool,
    selected: bool,
    loading: bool,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl std::fmt::Debug for IconButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IconButton")
            .field("id", &self.id)
            .field("icon", &self.icon)
            .field("tooltip", &self.tooltip)
            .finish_non_exhaustive()
    }
}

impl IconButton {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, icon: IconName, tooltip: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            tooltip: tooltip.into(),
            kind: ButtonKind::Ghost,
            size: ButtonSize::Medium,
            disabled: false,
            selected: false,
            loading: false,
            focus: None,
            on_click: None,
        }
    }

    #[must_use]
    pub const fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub const fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub const fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub const fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    #[must_use]
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
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

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut button = toolkit_button(self.id, self.kind, self.size)
            .icon(self.icon)
            .tooltip(self.tooltip.clone())
            .accessibility_label(self.tooltip)
            .loading(self.loading)
            .disabled(self.disabled)
            .selected(self.selected);
        if let Some(focus) = &self.focus {
            button = button.track_focus(focus);
        }
        if let Some(handler) = self.on_click {
            button = button.on_click(move |event, window, cx| handler(event, window, cx));
        }
        button
    }
}
