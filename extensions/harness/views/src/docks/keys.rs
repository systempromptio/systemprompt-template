//! The keys a dock answers to: Enter and Esc through the list bindings, A and R
//! as plain keys, and the feedback field.

use super::{DocksView, Head};
use gpui_kit::{Context, KeyDownEvent, Window};
use harness_engine::Reply;

impl DocksView {
    pub(super) fn primary(&self, window: &Window, cx: &mut Context<Self>) {
        if self.typing(window, cx) || !self.armed() {
            return;
        }
        match self.head(cx).map(|(head, _)| head) {
            Some(Head::Permission(ask)) => self.reply(&ask.id, Reply::Once, cx),
            Some(Head::Question(ask)) => self.submit_question(&ask, cx),
            None => {},
        }
    }

    pub(super) fn secondary(&self, window: &Window, cx: &mut Context<Self>) {
        if self.typing(window, cx) || !self.armed() {
            return;
        }
        match self.head(cx).map(|(head, _)| head) {
            Some(Head::Permission(ask)) => {
                self.reply(&ask.id, Reply::Reject { feedback: None }, cx);
            },
            Some(Head::Question(ask)) => self.dismiss_question(&ask.id, cx),
            None => {},
        }
    }

    pub(super) fn field_submitted(&self, cx: &mut Context<Self>) {
        match self.head(cx).map(|(head, _)| head) {
            Some(Head::Permission(ask)) if self.rejecting => self.reject_with_feedback(&ask.id, cx),
            Some(Head::Question(ask)) => self.submit_question(&ask, cx),
            _ => {},
        }
    }

    pub(super) fn field_escaped(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rejecting = false;
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn on_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modifiers = &event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.shift {
            return;
        }
        if self.typing(window, cx) || !self.armed() {
            return;
        }
        let Some((Head::Permission(ask), _)) = self.head(cx) else {
            return;
        };
        match event.keystroke.key.as_str() {
            "a" => self.reply(&ask.id, Reply::Always, cx),
            "r" => self.start_rejecting(window, cx),
            _ => return,
        }
        cx.stop_propagation();
    }
}
