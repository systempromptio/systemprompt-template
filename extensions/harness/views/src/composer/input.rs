//! How the composer reads the keyboard.
//!
//! Enter sends (or steers while a run is busy), Ctrl+Enter takes the other
//! delivery, Shift+Enter is a new line, and the arrow keys, Tab, Enter and
//! Escape drive an open completion list.

use gpui_kit::component::input::{
    Enter, Escape, IndentInline, InputEvent, MoveDown, MoveUp, TextareaState,
};
use gpui_kit::{Context, Div, Entity, InteractiveElement as _, SharedString, Window};
use systemprompt_engine::run::Delivery;

use crate::app::settings::DeliveryPref;
use crate::composer::ComposerView;
use crate::composer::drafts::{self, Attachment};
use crate::composer::popup::{self, Popup, PopupKind, Trigger};
use crate::state::Submission;

pub const fn delivery(preference: DeliveryPref, alternate: bool) -> Delivery {
    match (preference, alternate) {
        (DeliveryPref::Steer, false) | (DeliveryPref::Queue, true) => Delivery::Steer,
        (DeliveryPref::Queue, false) | (DeliveryPref::Steer, true) => Delivery::Queue,
    }
}

impl ComposerView {
    pub(super) fn on_input(
        &mut self,
        _input: &Entity<TextareaState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.refresh_popup(cx);
                cx.notify();
            },
            InputEvent::PressEnter { secondary, shift } if !*shift => {
                if self.enter_sends {
                    self.submit(*secondary, window, cx);
                } else if *secondary {
                    self.submit(false, window, cx);
                }
            },
            _ => {},
        }
    }

    pub(super) fn capture_keys(element: Div, cx: &Context<Self>) -> Div {
        element
            .capture_action(cx.listener(|this, _: &MoveUp, _, cx| {
                if this.step_popup(-1, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &MoveDown, _, cx| {
                if this.step_popup(1, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, action: &Enter, window, cx| {
                if !action.shift && !action.secondary && this.popup_has_items() {
                    this.accept_popup(None, window, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &IndentInline, window, cx| {
                if this.popup_has_items() {
                    this.accept_popup(None, window, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &Escape, _, cx| {
                if this.popup.take().is_some() {
                    cx.notify();
                    cx.stop_propagation();
                } else if this.is_busy(cx) {
                    this.stop(cx);
                    cx.stop_propagation();
                }
            }))
    }

    pub(super) fn submit(&mut self, alternate: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.has_model(cx) {
            return;
        }
        let preference = self
            .services
            .settings()
            .read(|settings| settings.composer.delivery);
        let submission = Submission {
            text: self.text(cx).trim_end().to_owned(),
            attachments: self
                .attachments
                .iter()
                .map(|attachment| attachment.input.clone())
                .collect(),
            delivery: delivery(preference, alternate),
        };
        if submission.is_empty() {
            return;
        }
        self.app.update(cx, |app, cx| app.send(submission, cx));
        self.attachments.clear();
        self.popup = None;
        self.set_text(String::new(), window, cx);
        cx.notify();
    }

    pub(super) fn stop(&self, cx: &mut Context<Self>) {
        if !self
            .app
            .update(cx, crate::state::app_state::AppState::abort_current)
        {
            tracing::trace!("no run to stop");
        }
    }

    fn popup_has_items(&self) -> bool {
        self.popup
            .as_ref()
            .is_some_and(|popup| !popup.items.is_empty())
    }

    fn step_popup(&mut self, delta: isize, cx: &mut Context<Self>) -> bool {
        let Some(popup) = self.popup.as_mut().filter(|popup| !popup.items.is_empty()) else {
            return false;
        };
        popup.step(delta);
        cx.notify();
        true
    }

    fn refresh_popup(&mut self, cx: &Context<Self>) {
        let (value, cursor) = {
            let input = self.input.read(cx);
            (input.value().to_string(), input.cursor())
        };
        self.popup = match popup::trigger(&value, cursor) {
            Some(Trigger::Slash { query }) => Some(Popup {
                kind: PopupKind::Slash,
                items: popup::command_items(&query, &self.app.read(cx).commands),
                selected: 0,
            }),
            Some(Trigger::Mention { start, query }) => Some(Popup {
                kind: PopupKind::Mention { start },
                items: popup::path_items(&query, &self.files),
                selected: 0,
            }),
            None => None,
        };
    }

    pub(super) fn accept_popup(
        &mut self,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(open) = self.popup.take() else {
            return;
        };
        let chosen = match index {
            Some(index) => open.items.get(index).cloned(),
            None => open.current().cloned(),
        };
        let Some(item) = chosen else {
            cx.notify();
            return;
        };
        match open.kind {
            PopupKind::Slash => self.set_text(format!("/{} ", item.value), window, cx),
            PopupKind::Mention { start } => {
                let (value, cursor) = {
                    let input = self.input.read(cx);
                    (input.value().to_string(), input.cursor())
                };
                let (next, caret) = popup::insert_mention(&value, start, cursor, &item.value);
                self.input.update(cx, |input, cx| {
                    input.set_value(SharedString::from(next), window, cx);
                    input.set_selected_range(caret..caret, cx);
                });
                drafts::add(&mut self.attachments, [Attachment::mention(&item.value)]);
            },
        }
        cx.notify();
    }
}
