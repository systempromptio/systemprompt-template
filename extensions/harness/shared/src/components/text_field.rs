//! A labelled single-line text field with hint or error text, password masking,
//! and change and submit callbacks.

use std::rc::Rc;

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window, div,
};

use crate::theme::{space, text, tokens};

type TextHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A text field entity. Create it with `cx.new(|cx| TextField::new(window,
/// cx).label(..))`.
///
/// `on_change` runs with the new text after every edit, `on_submit` when Enter
/// is pressed. `set_text` does not fire `on_change`. An error set with
/// `set_error` replaces the hint and tints the border.
pub struct TextField {
    input: Entity<InputState>,
    label: Option<SharedString>,
    hint: Option<SharedString>,
    error: Option<SharedString>,
    placeholder: Option<SharedString>,
    applied_placeholder: Option<SharedString>,
    pending_value: Option<SharedString>,
    password: bool,
    applied_password: bool,
    disabled: bool,
    on_change: Option<TextHandler>,
    on_submit: Option<TextHandler>,
    _subscription: Subscription,
}

impl std::fmt::Debug for TextField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextField")
            .field("label", &self.label)
            .field("error", &self.error)
            .field("password", &self.password)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl TextField {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe_in(&input, window, Self::on_input_event);
        Self {
            input,
            label: None,
            hint: None,
            error: None,
            placeholder: None,
            applied_placeholder: None,
            pending_value: None,
            password: false,
            applied_password: false,
            disabled: false,
            on_change: None,
            on_submit: None,
            _subscription: subscription,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.pending_value = Some(value.into());
        self
    }

    #[must_use]
    pub const fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn on_submit(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn text(&self, cx: &App) -> SharedString {
        self.input.read(cx).value()
    }

    #[expect(
        clippy::needless_pass_by_ref_mut,
        reason = "the toolkit input takes the window mutably inside the update closure, which the lint does not follow"
    )]
    pub fn set_text(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        self.input
            .update(cx, |input, cx| input.set_value(value, window, cx));
        cx.notify();
    }

    pub fn set_error(&mut self, error: Option<SharedString>, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        let handle = self.input.read(cx).focus_handle(cx);
        handle.focus(window, cx);
    }

    #[must_use]
    pub const fn input(&self) -> &Entity<InputState> {
        &self.input
    }

    fn on_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let handler = match event {
            InputEvent::Change => self.on_change.clone(),
            InputEvent::PressEnter { .. } => self.on_submit.clone(),
            _ => None,
        };
        if let Some(handler) = handler {
            let value = state.read(cx).value();
            handler(&value, window, cx);
        }
    }

    fn sync_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.applied_placeholder != self.placeholder {
            let placeholder = self.placeholder.clone().unwrap_or_default();
            self.input.update(cx, |input, cx| {
                input.set_placeholder(placeholder, window, cx);
            });
            self.applied_placeholder = self.placeholder.clone();
        }
        if self.applied_password != self.password {
            let masked = self.password;
            self.input
                .update(cx, |input, cx| input.set_masked(masked, window, cx));
            self.applied_password = masked;
        }
        if let Some(value) = self.pending_value.take() {
            self.input
                .update(cx, |input, cx| input.set_value(value, window, cx));
        }
    }
}

impl Focusable for TextField {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_input(window, cx);
        let palette = *tokens(cx);
        let invalid = self.error.is_some();

        let mut field = Input::new(&self.input).disabled(self.disabled);
        if self.password {
            field = field.mask_toggle();
        }
        if invalid {
            field = field.border_color(palette.danger.fg);
        }
        if let Some(label) = &self.label {
            field = field.aria_label(label.clone());
        }

        let note = match (&self.error, &self.hint) {
            (Some(error), _) => Some((error.clone(), palette.danger.fg)),
            (None, Some(hint)) => Some((hint.clone(), palette.text.muted)),
            (None, None) => None,
        };

        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .children(self.label.clone().map(|label| {
                div()
                    .text_size(text::SMALL)
                    .text_color(palette.text.secondary)
                    .child(label)
            }))
            .child(field)
            .children(
                note.map(|(note, color)| {
                    div().text_size(text::SMALL).text_color(color).child(note)
                }),
            )
    }
}
