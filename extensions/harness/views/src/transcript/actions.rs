//! What a message offers by pointer or keyboard: copy it, undo the session back
//! to it, fork a new session from it, and the list selection that the keyboard
//! moves.

use gpui_kit::{
    ClipboardItem, Context, Div, InteractiveElement as _, KeyDownEvent, Stateful, Window,
};
use harness_store::{Message, MessageWithParts, PartBody};
use harness_ui::components::{
    Activate, Dismiss, SelectFirst, SelectLast, SelectNext, SelectPrevious, Toast,
};

use crate::strings::transcript as text;
use crate::views::transcript::{TranscriptEvent, TranscriptView, rows};

impl TranscriptView {
    pub(super) fn bind_keys(root: Stateful<Div>, cx: &Context<Self>) -> Stateful<Div> {
        root.on_action(cx.listener(|this, _: &SelectNext, _, cx| this.step(1, cx)))
            .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| this.step(-1, cx)))
            .on_action(cx.listener(|this, _: &SelectFirst, _, cx| this.jump(Some(0), cx)))
            .on_action(cx.listener(|this, _: &SelectLast, _, cx| {
                let last = this.rows.len().checked_sub(1);
                this.jump(last, cx);
            }))
            .on_action(cx.listener(|this, _: &Activate, _, cx| this.toggle_selected(cx)))
            .on_action(cx.listener(|this, _: &Dismiss, _, cx| {
                if this.selected.take().is_some() {
                    cx.notify();
                } else {
                    cx.propagate();
                }
            }))
            .on_key_down(cx.listener(Self::on_key))
    }

    pub(super) fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(index);
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(last) = self.rows.len().checked_sub(1) else {
            return;
        };
        let next = self.selected.map_or(last, |current| {
            current.saturating_add_signed(delta).min(last)
        });
        self.jump(Some(next), cx);
    }

    fn jump(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        self.selected = index;
        if let Some(index) = index {
            self.list.scroll_to_reveal_item(index);
        }
        cx.notify();
    }

    fn toggle_selected(&mut self, cx: &mut Context<Self>) {
        let Some(entry) = self.selected.and_then(|index| self.entry(index, cx)) else {
            return;
        };
        let ids: Vec<_> = entry
            .parts
            .iter()
            .filter(|part| {
                matches!(
                    part.body,
                    PartBody::Tool { .. } | PartBody::Reasoning { .. }
                )
            })
            .map(|part| part.id.clone())
            .collect();
        let open = ids.iter().any(|id| !self.expanded.contains(id));
        for id in ids {
            if open {
                self.expanded.insert(id);
            } else {
                self.expanded.remove(&id);
            }
        }
        if let Some(index) = self.selected {
            self.list.remeasure_items(index..index + 1);
        }
        cx.notify();
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.selected else { return };
        let modifiers = &event.keystroke.modifiers;
        if modifiers.alt || modifiers.platform || modifiers.shift {
            return;
        }
        match (event.keystroke.key.as_str(), modifiers.control) {
            ("c", _) => self.copy_message(index, window, cx),
            ("r", false) => self.revert_message(index, cx),
            ("f", false) => self.fork_message(index, cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    fn entry(&self, index: usize, cx: &Context<Self>) -> Option<MessageWithParts> {
        let key = self.rows.get(index)?;
        self.session
            .read(cx)
            .model()
            .visible_messages()
            .get(key.index)
            .cloned()
    }

    pub(super) fn copy_message(&self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.entry(index, cx) else {
            return;
        };
        let body = rows::plain_text(&entry);
        if body.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(body));
        Toast::success(text::COPIED).show(window, cx);
    }

    pub(super) fn revert_message(&self, index: usize, cx: &mut Context<Self>) {
        if self.session.read(cx).model().is_busy() {
            return;
        }
        let Some(entry) = self.entry(index, cx) else {
            return;
        };
        let Message::User(user) = &entry.message else {
            return;
        };
        let id = user.id.clone();
        let body = rows::plain_text(&entry);
        if !body.is_empty() {
            cx.emit(TranscriptEvent::RestoreDraft(body));
        }
        self.session.update(cx, |state, cx| state.revert(id, cx));
    }

    pub(super) fn fork_message(&self, index: usize, cx: &mut Context<Self>) {
        let Some(key) = self.rows.get(index).cloned() else {
            return;
        };
        let before =
            rows::following_message(self.session.read(cx).model().visible_messages(), key.index);
        self.session.update(cx, |state, cx| state.fork(before, cx));
    }
}
