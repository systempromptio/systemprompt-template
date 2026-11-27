//! Inline banners and transient toasts.

use std::rc::Rc;

use gpui_kit::component::button::{Button as ToolkitButton, ButtonVariants as _};
use gpui_kit::component::notification::{Notification, NotificationType};
use gpui_kit::component::{Sizable as _, WindowExt as _};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window,
    div,
};

use super::{ActionHandler, Button, ButtonSize, IconButton};
use crate::icons::{Icon, IconName};
use crate::theme::{Tone, radius, size, space, text, tokens};

const fn tone_icon(tone: Tone) -> IconName {
    match tone {
        Tone::Success => IconName::CircleCheck,
        Tone::Warning => IconName::Alert,
        Tone::Danger => IconName::CircleX,
        Tone::Neutral | Tone::Accent | Tone::Info => IconName::Info,
    }
}

/// A message across the top of a view or panel: a tone, optional title,
/// optional action and dismiss button.
#[derive(IntoElement)]
pub struct Banner {
    id: ElementId,
    tone: Tone,
    title: Option<SharedString>,
    message: SharedString,
    action: Option<(SharedString, ActionHandler)>,
    dismiss: Option<(SharedString, ActionHandler)>,
}

impl std::fmt::Debug for Banner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Banner")
            .field("id", &self.id)
            .field("tone", &self.tone)
            .field("title", &self.title)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl Banner {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            tone,
            title: None,
            message: message.into(),
            action: None,
            dismiss: None,
        }
    }

    #[must_use]
    pub fn info(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self::new(id, Tone::Info, message)
    }

    #[must_use]
    pub fn warning(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self::new(id, Tone::Warning, message)
    }

    #[must_use]
    pub fn error(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self::new(id, Tone::Danger, message)
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    #[must_use]
    pub fn dismissible(
        mut self,
        tooltip: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.dismiss = Some((tooltip.into(), Rc::new(handler)));
        self
    }
}

impl RenderOnce for Banner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = tokens(cx).tone(self.tone);
        let primary = tokens(cx).text.primary;
        let id = self.id;
        div()
            .flex()
            .flex_row()
            .items_start()
            .gap(space::MD)
            .w_full()
            .px(space::LG)
            .py(space::MD)
            .rounded(radius::LG)
            .border_1()
            .border_color(colors.border)
            .bg(colors.bg)
            .text_size(text::BODY)
            .child(
                div().flex_shrink_0().pt(space::XXS).child(
                    Icon::new(tone_icon(self.tone))
                        .size(size::ICON_MD)
                        .color(colors.fg),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(space::XXS)
                    .children(self.title.map(|title| {
                        div()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .text_color(colors.fg)
                            .child(title)
                    }))
                    .child(div().text_color(primary).child(self.message)),
            )
            .children(self.action.map(|(label, handler)| {
                Button::new(ElementId::NamedChild(
                    std::sync::Arc::new(id.clone()),
                    "action".into(),
                ))
                .label(label)
                .size(ButtonSize::Small)
                .on_click(move |_, window, cx| handler(window, cx))
            }))
            .children(self.dismiss.map(|(tooltip, handler)| {
                IconButton::new(
                    ElementId::NamedChild(std::sync::Arc::new(id.clone()), "dismiss".into()),
                    IconName::X,
                    tooltip,
                )
                .small()
                .on_click(move |_, window, cx| handler(window, cx))
            }))
    }
}

/// A transient notification in the window's toast stack, pushed with `show`.
///
/// Without an action it hides itself after a few seconds; with one it stays
/// until acted on (which dismisses it) or closed. `sticky()` never auto-hides.
pub struct Toast {
    tone: Tone,
    title: Option<SharedString>,
    message: SharedString,
    action: Option<(SharedString, ActionHandler)>,
    sticky: bool,
}

impl std::fmt::Debug for Toast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toast")
            .field("tone", &self.tone)
            .field("title", &self.title)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl Toast {
    #[must_use]
    pub fn new(tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            tone,
            title: None,
            message: message.into(),
            action: None,
            sticky: false,
        }
    }

    #[must_use]
    pub fn info(message: impl Into<SharedString>) -> Self {
        Self::new(Tone::Info, message)
    }

    #[must_use]
    pub fn success(message: impl Into<SharedString>) -> Self {
        Self::new(Tone::Success, message)
    }

    #[must_use]
    pub fn warning(message: impl Into<SharedString>) -> Self {
        Self::new(Tone::Warning, message)
    }

    #[must_use]
    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::new(Tone::Danger, message)
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    #[must_use]
    pub const fn sticky(mut self) -> Self {
        self.sticky = true;
        self
    }

    pub fn show(self, window: &mut Window, cx: &mut App) {
        let kind = match self.tone {
            Tone::Success => NotificationType::Success,
            Tone::Warning => NotificationType::Warning,
            Tone::Danger => NotificationType::Error,
            Tone::Neutral | Tone::Accent | Tone::Info => NotificationType::Info,
        };
        let mut note = Notification::new()
            .message(self.message)
            .with_type(kind)
            .icon(tone_icon(self.tone))
            .autohide(!self.sticky);
        if let Some(title) = self.title {
            note = note.title(title);
        }
        if let Some((label, handler)) = self.action {
            note = note.action(move |_note, _window, cx| {
                let handler = Rc::clone(&handler);
                let toast = cx.weak_entity();
                ToolkitButton::new("toast-action")
                    .primary()
                    .small()
                    .label(label.clone())
                    .on_click(move |_, window, cx| {
                        handler(window, cx);
                        if toast
                            .update(cx, |note, cx| note.dismiss(window, cx))
                            .is_err()
                        {
                            tracing::debug!("toast closed before its action finished");
                        }
                    })
            });
        }
        window.push_notification(note, cx);
    }
}
