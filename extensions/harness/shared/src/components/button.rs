//! Labelled buttons in four kinds and three sizes, and the toolkit button both
//! button types build on.

use std::rc::Rc;

use gpui_kit::component::button::{Button as ToolkitButton, ButtonVariants as _};
use gpui_kit::component::{Disableable as _, Selectable as _, Sizable as _};
use gpui_kit::{
    App, ClickEvent, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, Styled as _,
    Window,
};

use super::ClickHandler;
use crate::icons::IconName;

/// The emphasis of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonKind {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Danger,
}

/// The height class of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

pub(super) fn toolkit_button(id: ElementId, kind: ButtonKind, size: ButtonSize) -> ToolkitButton {
    let button = ToolkitButton::new(id);
    let button = match kind {
        ButtonKind::Primary => button.primary(),
        ButtonKind::Secondary => button.outline(),
        ButtonKind::Ghost => button.ghost(),
        ButtonKind::Danger => button.danger(),
    };
    match size {
        ButtonSize::Small => button.small(),
        ButtonSize::Medium => button,
        ButtonSize::Large => button.large(),
    }
}

/// A labelled button. Enter and Space activate it when focused; the focus ring
/// shows for keyboard focus.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<IconName>,
    kind: ButtonKind,
    size: ButtonSize,
    loading: bool,
    disabled: bool,
    selected: bool,
    full_width: bool,
    tooltip: Option<SharedString>,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("kind", &self.kind)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

impl Button {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            kind: ButtonKind::default(),
            size: ButtonSize::default(),
            loading: false,
            disabled: false,
            selected: false,
            full_width: false,
            tooltip: None,
            focus: None,
            on_click: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub const fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub const fn primary(self) -> Self {
        self.kind(ButtonKind::Primary)
    }

    #[must_use]
    pub const fn ghost(self) -> Self {
        self.kind(ButtonKind::Ghost)
    }

    #[must_use]
    pub const fn danger(self) -> Self {
        self.kind(ButtonKind::Danger)
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
    pub const fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }

    #[must_use]
    pub const fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
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
    pub const fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
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

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut button = toolkit_button(self.id, self.kind, self.size)
            .loading(self.loading)
            .disabled(self.disabled)
            .selected(self.selected);
        if let Some(label) = self.label {
            button = button.label(label);
        }
        if let Some(icon) = self.icon {
            button = button.icon(icon);
        }
        if let Some(tooltip) = self.tooltip {
            button = button.tooltip(tooltip);
        }
        if let Some(focus) = &self.focus {
            button = button.track_focus(focus);
        }
        if let Some(handler) = self.on_click {
            button = button.on_click(move |event, window, cx| handler(event, window, cx));
        }
        if self.full_width {
            button = button.w_full();
        }
        button
    }
}
