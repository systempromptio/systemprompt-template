//! A modal dialog scaffold: title, optional description and body, footer
//! actions; Escape cancels, Enter confirms.

use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement as _, Pixels, SharedString, Styled as _, Window,
    div, px,
};

use super::{Button, ButtonKind};
use crate::theme::{space, text, tokens};

type PressHandler = Rc<dyn Fn(&mut Window, &mut App) -> bool>;
type BodyBuilder = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type DismissHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A footer button of a dialog. Its handler returns whether the dialog should
/// close.
#[derive(Clone)]
pub struct DialogAction {
    label: SharedString,
    kind: ButtonKind,
    disabled: bool,
    loading: bool,
    on_press: PressHandler,
}

impl std::fmt::Debug for DialogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogAction")
            .field("label", &self.label)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl DialogAction {
    #[must_use]
    pub fn new(
        label: impl Into<SharedString>,
        on_press: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            kind: ButtonKind::Secondary,
            disabled: false,
            loading: false,
            on_press: Rc::new(on_press),
        }
    }

    #[must_use]
    pub const fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
}

/// Everything a dialog shows. Build it, then [`DialogSpec::open`] it on a
/// window.
///
/// Enter runs the confirm action and Escape the cancel action; an action's
/// handler returns whether to close. The body builder runs on every render, so
/// create entities before opening and capture them. `on_dismiss` runs
/// whenever the dialog closes.
#[derive(Clone)]
pub struct DialogSpec {
    title: SharedString,
    description: Option<SharedString>,
    body: Option<BodyBuilder>,
    confirm: Option<DialogAction>,
    cancel: Option<DialogAction>,
    width: Pixels,
    close_on_overlay: bool,
    on_dismiss: Option<DismissHandler>,
}

impl std::fmt::Debug for DialogSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogSpec")
            .field("title", &self.title)
            .field("confirm", &self.confirm)
            .field("cancel", &self.cancel)
            .finish_non_exhaustive()
    }
}

impl DialogSpec {
    #[must_use]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            body: None,
            confirm: None,
            cancel: None,
            width: px(440.),
            close_on_overlay: true,
            on_dismiss: None,
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn body<E: IntoElement>(
        mut self,
        body: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.body = Some(Rc::new(move |window, cx| {
            body(window, cx).into_any_element()
        }));
        self
    }

    #[must_use]
    pub fn confirm(mut self, action: DialogAction) -> Self {
        self.confirm = Some(action);
        self
    }

    #[must_use]
    pub fn cancel(mut self, action: DialogAction) -> Self {
        self.cancel = Some(action);
        self
    }

    #[must_use]
    pub const fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn close_on_overlay(mut self, close: bool) -> Self {
        self.close_on_overlay = close;
        self
    }

    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    pub fn open(self, window: &mut Window, cx: &mut App) {
        window.open_dialog(cx, move |dialog, window, cx| {
            let palette = *tokens(cx);
            let spec = &self;
            let mut dialog = dialog
                .title(
                    div()
                        .text_size(text::TITLE)
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .text_color(palette.text.primary)
                        .child(spec.title.clone()),
                )
                .w(spec.width)
                .overlay_closable(spec.close_on_overlay)
                .keyboard(true);
            if let Some(confirm) = &spec.confirm {
                let press = Rc::clone(&confirm.on_press);
                let enabled = !confirm.disabled && !confirm.loading;
                dialog = dialog.on_ok(move |_, window, cx| enabled && press(window, cx));
            }
            if let Some(cancel) = &spec.cancel {
                let press = Rc::clone(&cancel.on_press);
                dialog = dialog.on_cancel(move |_, window, cx| press(window, cx));
            }
            if let Some(dismiss) = spec.on_dismiss.clone() {
                dialog = dialog.on_close(move |_, window, cx| dismiss(window, cx));
            }

            let body = spec.body.as_ref().map(|build| build(window, cx));
            dialog = dialog.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::LG)
                    .text_size(text::BODY)
                    .children(spec.description.clone().map(|description| {
                        div().text_color(palette.text.secondary).child(description)
                    }))
                    .children(body),
            );

            let footer_button = |id: &'static str, action: &DialogAction| {
                let press = Rc::clone(&action.on_press);
                Button::new(id)
                    .label(action.label.clone())
                    .kind(action.kind)
                    .disabled(action.disabled)
                    .loading(action.loading)
                    .on_click(move |_, window, cx| {
                        if press(window, cx) {
                            window.close_dialog(cx);
                        }
                    })
            };
            if spec.confirm.is_some() || spec.cancel.is_some() {
                dialog = dialog.footer(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(space::MD)
                        .w_full()
                        .children(
                            spec.cancel
                                .as_ref()
                                .map(|action| footer_button("dialog-cancel", action)),
                        )
                        .children(
                            spec.confirm
                                .as_ref()
                                .map(|action| footer_button("dialog-confirm", action)),
                        ),
                );
            }
            dialog
        });
    }
}
