//! A menu: a trigger button that opens a keyboard-navigable list of commands.

use std::rc::Rc;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button as ToolkitButton, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::{Anchor, App, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window, px};

use super::ActionHandler;
use crate::icons::IconName;

pub(super) const MENU_MIN_WIDTH: Pixels = px(180.);

/// One line of a [`Menu`].
#[derive(Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Label(SharedString),
    Separator,
}

impl std::fmt::Debug for MenuEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(item) => f.debug_tuple("Item").field(&item.label).finish(),
            Self::Label(label) => f.debug_tuple("Label").field(label).finish(),
            Self::Separator => f.write_str("Separator"),
        }
    }
}

/// A command in a [`Menu`].
#[derive(Clone)]
pub struct MenuItem {
    label: SharedString,
    icon: Option<IconName>,
    checked: bool,
    disabled: bool,
    on_select: ActionHandler,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("label", &self.label)
            .field("checked", &self.checked)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl MenuItem {
    #[must_use]
    pub fn new(
        label: impl Into<SharedString>,
        on_select: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            icon: None,
            checked: false,
            disabled: false,
            on_select: Rc::new(on_select),
        }
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
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
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}

enum Trigger {
    Icon {
        icon: IconName,
        tooltip: SharedString,
    },
    Label(SharedString),
}

/// A trigger button that opens a list of commands. Arrow keys move, Enter runs,
/// Escape closes.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    trigger: Trigger,
    entries: Rc<[MenuEntry]>,
    align_end: bool,
}

impl std::fmt::Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("id", &self.id)
            .field("entries", &self.entries)
            .finish_non_exhaustive()
    }
}

impl Menu {
    #[must_use]
    pub fn icon(
        id: impl Into<ElementId>,
        icon: IconName,
        tooltip: impl Into<SharedString>,
        entries: impl IntoIterator<Item = MenuEntry>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: Trigger::Icon {
                icon,
                tooltip: tooltip.into(),
            },
            entries: entries.into_iter().collect(),
            align_end: false,
        }
    }

    #[must_use]
    pub fn labeled(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        entries: impl IntoIterator<Item = MenuEntry>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: Trigger::Label(label.into()),
            entries: entries.into_iter().collect(),
            align_end: false,
        }
    }

    #[must_use]
    pub const fn align_end(mut self) -> Self {
        self.align_end = true;
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let trigger = match self.trigger {
            Trigger::Icon { icon, tooltip } => ToolkitButton::new(self.id)
                .ghost()
                .small()
                .icon(icon)
                .tooltip(tooltip.clone())
                .accessibility_label(tooltip),
            Trigger::Label(label) => ToolkitButton::new(self.id)
                .ghost()
                .small()
                .label(label)
                .dropdown_caret(true),
        };
        let anchor = if self.align_end {
            Anchor::TopRight
        } else {
            Anchor::TopLeft
        };
        let entries = self.entries;
        trigger.dropdown_menu_with_anchor(anchor, move |menu: PopupMenu, _window, _cx| {
            let mut menu = menu.min_w(MENU_MIN_WIDTH);
            for entry in entries.iter() {
                menu = match entry {
                    MenuEntry::Separator => menu.separator(),
                    MenuEntry::Label(label) => menu.label(label.clone()),
                    MenuEntry::Item(item) => {
                        let handler = Rc::clone(&item.on_select);
                        let mut row = PopupMenuItem::new(item.label.clone())
                            .checked(item.checked)
                            .disabled(item.disabled)
                            .on_click(move |_, window, cx| handler(window, cx));
                        if let Some(icon) = item.icon {
                            row = row.icon(icon);
                        }
                        menu.item(row)
                    },
                };
            }
            menu
        })
    }
}
