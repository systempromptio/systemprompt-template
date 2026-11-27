//! A select: pick one value from a list in a keyboard-navigable popover.

use std::rc::Rc;

use gpui_kit::component::button::Button as ToolkitButton;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{
    App, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Styled as _, Window, px,
};

use super::menu::MENU_MIN_WIDTH;
use crate::icons::IconName;

type SelectHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// One choice of a [`Select`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub disabled: bool,
}

impl SelectOption {
    #[must_use]
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A button showing the current choice that opens the list of options. Arrow
/// keys move, Enter picks, Escape closes.
///
/// `on_select` receives the picked option's value; `placeholder` shows while
/// nothing is selected.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    options: Rc<[SelectOption]>,
    selected: Option<SharedString>,
    placeholder: Option<SharedString>,
    disabled: bool,
    small: bool,
    width: Option<Pixels>,
    handler: Option<SelectHandler>,
}

impl std::fmt::Debug for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select")
            .field("id", &self.id)
            .field("options", &self.options.len())
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl Select {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, options: impl IntoIterator<Item = SelectOption>) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().collect(),
            selected: None,
            placeholder: None,
            disabled: false,
            small: false,
            width: None,
            handler: None,
        }
    }

    #[must_use]
    pub fn selected(mut self, value: Option<SharedString>) -> Self {
        self.selected = value;
        self
    }

    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
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
    pub const fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let current = self
            .selected
            .as_ref()
            .and_then(|value| self.options.iter().find(|option| &option.value == value));
        let label = current
            .map(|option| option.label.clone())
            .or_else(|| self.placeholder.clone())
            .unwrap_or_default();
        let icon = current.and_then(|option| option.icon);

        let mut trigger = ToolkitButton::new(self.id)
            .outline()
            .label(label)
            .dropdown_caret(true)
            .disabled(self.disabled);
        if let Some(icon) = icon {
            trigger = trigger.icon(icon);
        }
        if self.small {
            trigger = trigger.small();
        }
        if let Some(width) = self.width {
            trigger = trigger.w(width);
        }

        let options = self.options;
        let selected = self.selected;
        let on_select = self.handler;
        let min_width = self.width.unwrap_or(MENU_MIN_WIDTH);
        trigger.dropdown_menu(move |menu: PopupMenu, _window, _cx| {
            let mut menu = menu.min_w(min_width).max_h(px(320.)).scrollable(true);
            for option in options.iter() {
                let mut item = PopupMenuItem::new(option.label.clone())
                    .checked(selected.as_ref() == Some(&option.value))
                    .disabled(option.disabled);
                if let Some(icon) = option.icon {
                    item = item.icon(icon);
                }
                if let Some(handler) = on_select.clone() {
                    let value = option.value.clone();
                    item = item.on_click(move |_, window, cx| handler(&value, window, cx));
                }
                menu = menu.item(item);
            }
            menu
        })
    }
}
