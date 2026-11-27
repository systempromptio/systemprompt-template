//! Checkbox and on/off toggle, controlled by the caller: they report the
//! requested value and render what they are given.

use std::rc::Rc;

use gpui_kit::component::checkbox::Checkbox as ToolkitCheckbox;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A checkbox with an optional label. Space toggles it when focused.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    small: bool,
    on_change: Option<ToggleHandler>,
}

impl std::fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("checked", &self.checked)
            .finish_non_exhaustive()
    }
}

impl Checkbox {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked: false,
            disabled: false,
            small: false,
            on_change: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub const fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn small(mut self) -> Self {
        self.small = true;
        self
    }

    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut checkbox = ToolkitCheckbox::new(self.id)
            .checked(self.checked)
            .disabled(self.disabled);
        if let Some(label) = self.label {
            checkbox = checkbox.label(label);
        }
        if self.small {
            checkbox = checkbox.small();
        }
        if let Some(handler) = self.on_change {
            checkbox =
                checkbox.on_change(move |checked: &bool, window, cx| handler(*checked, window, cx));
        }
        checkbox
    }
}

/// An on/off switch with an optional label, for settings that apply
/// immediately.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    label: Option<SharedString>,
    on: bool,
    disabled: bool,
    small: bool,
    on_change: Option<ToggleHandler>,
}

impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggle")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("on", &self.on)
            .finish_non_exhaustive()
    }
}

impl Toggle {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            on: false,
            disabled: false,
            small: false,
            on_change: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn small(mut self) -> Self {
        self.small = true;
        self
    }

    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut switch = Switch::new(self.id)
            .checked(self.on)
            .disabled(self.disabled);
        if let Some(label) = self.label {
            switch = switch.label(label);
        }
        if self.small {
            switch = switch.small();
        }
        if let Some(handler) = self.on_change {
            switch = switch.on_change(move |on: &bool, window: &mut Window, cx: &mut App| {
                handler(*on, window, cx);
            });
        }
        switch
    }
}
