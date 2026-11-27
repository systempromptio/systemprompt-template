//! The non-tool parts of a reply: streamed markdown text, collapsible
//! reasoning, the compaction marker, the summary a compaction leaves, attached
//! files and subtask starts.

use gpui_kit::{
    AnyElement, Context, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div,
};
use harness_store::{MessageId, MessageWithParts, Part, PartBody, PartId, SessionId};
use harness_ui::components::{Button, Divider, Spinner};
use harness_ui::theme::{radius, size, space, text as type_scale, tokens};
use harness_ui::{Icon, IconName, Markdown};

use crate::strings::transcript as text;
use crate::views::transcript::message::file_chip;
use crate::views::transcript::{TranscriptEvent, TranscriptView};

pub(super) fn part_element_id(prefix: &str, id: &PartId) -> ElementId {
    ElementId::Name(SharedString::from(format!("{prefix}-{id}")))
}

impl TranscriptView {
    pub(super) fn render_part(
        &mut self,
        part: &Part,
        streaming: Option<&PartId>,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        let live = streaming == Some(&part.id);
        match &part.body {
            PartBody::Text {
                text,
                synthetic: false,
            } if !text.trim().is_empty() => Some(self.render_text(&part.id, text, live, false)),
            PartBody::Reasoning { text, .. } if !text.trim().is_empty() => {
                Some(self.render_reasoning(part, text, live, cx))
            },
            PartBody::Tool { tool, state, .. } => Some(self.render_tool(part, tool, state, cx)),
            PartBody::File { name, url, .. } => Some(file_chip(name.as_deref(), url, cx)),
            PartBody::Compaction { auto } => Some(compaction_divider(*auto)),
            PartBody::Subtask {
                agent,
                description,
                child_session,
                ..
            } => Some(self.render_subtask(
                &part.id,
                (agent.as_str(), description.as_str()),
                child_session.as_ref(),
                cx,
            )),
            _ => None,
        }
    }

    pub(super) fn render_text(
        &mut self,
        id: &PartId,
        source: &str,
        streaming: bool,
        muted: bool,
    ) -> AnyElement {
        let document = self.documents.document(id, source);
        Markdown::new(part_element_id("markdown", id), document)
            .streaming(streaming)
            .muted(muted)
            .copy_button(text::COPY_CODE)
            .into_any_element()
    }

    pub(super) fn toggle_part(&mut self, id: &PartId, message: &MessageId, cx: &mut Context<Self>) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.clone());
        }
        if let Some(row) = self.rows.iter().position(|row| &row.id == message) {
            self.list.remeasure_items(row..row + 1);
        }
        cx.notify();
    }

    fn render_reasoning(
        &mut self,
        part: &Part,
        source: &str,
        live: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let open = self.expanded.contains(&part.id);
        let label = if live {
            text::THINKING_LIVE
        } else {
            text::THINKING
        };
        let header = Self::disclosure(part, label, open, cx);
        let mut column = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .child(header);
        if open {
            let body = self.render_text(&part.id, source, live, true);
            column = column.child(div().pl(space::XL).child(body));
        }
        column.into_any_element()
    }

    pub(super) fn render_summary(
        &mut self,
        entry: &MessageWithParts,
        cx: &Context<Self>,
    ) -> AnyElement {
        let Some((part, source)) = entry.parts.iter().find_map(|part| match &part.body {
            PartBody::Text { text, .. } if !text.trim().is_empty() => {
                Some((part.clone(), text.clone()))
            },
            _ => None,
        }) else {
            return compaction_divider(true);
        };
        let open = self.expanded.contains(&part.id);
        let header = Self::disclosure(&part, text::SUMMARY, open, cx);
        let mut column = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .child(header);
        if open {
            let body = self.render_text(&part.id, &source, false, true);
            column = column.child(div().pl(space::XL).child(body));
        }
        column.into_any_element()
    }

    pub(super) fn render_compaction_note(
        entry: &MessageWithParts,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        let auto = entry
            .parts
            .iter()
            .any(|part| matches!(part.body, PartBody::Compaction { auto: true }));
        compaction_divider(auto)
    }

    fn disclosure(part: &Part, label: &'static str, open: bool, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let icon = if open {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        };
        let (id, message) = (part.id.clone(), part.message_id.clone());
        div()
            .id(part_element_id("disclosure", &part.id))
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .cursor_pointer()
            .text_size(type_scale::SMALL)
            .text_color(palette.text.muted)
            .hover(|style| style.text_color(palette.text.secondary))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_part(&id, &message, cx)))
            .child(Icon::new(icon).size(size::ICON_SM))
            .child(div().italic().child(label))
            .into_any_element()
    }

    pub(super) fn render_subtask(
        &self,
        id: &PartId,
        (agent, description): (&str, &str),
        child: Option<&SessionId>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let progress = child.and_then(|child| self.children.progress(child));
        let line = progress.map_or_else(
            || text::SUBTASK_STARTING.to_owned(),
            |progress| text::subtask_progress(progress.steps, progress.current.as_deref()),
        );
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .w_full()
            .px(space::MD)
            .py(space::XS)
            .rounded(radius::MD)
            .border_1()
            .border_color(palette.border.subtle)
            .text_size(type_scale::SMALL)
            .child(
                Icon::new(IconName::Branch)
                    .size(size::ICON_SM)
                    .color(palette.text.secondary),
            )
            .child(
                div()
                    .text_color(palette.text.primary)
                    .child(SharedString::from(agent.to_owned())),
            )
            .child(
                div()
                    .truncate()
                    .text_color(palette.text.secondary)
                    .child(SharedString::from(description.to_owned())),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(palette.text.muted)
                    .child(SharedString::from(line)),
            );
        if let Some(child) = child.cloned() {
            row = row.child(
                Button::new(part_element_id("subtask-open", id))
                    .label(text::OPEN_SUBTASK)
                    .small()
                    .ghost()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(TranscriptEvent::OpenSession(child.clone()));
                    })),
            );
        } else {
            row = row.child(Spinner::new().size(size::ICON_SM));
        }
        row.into_any_element()
    }
}

fn compaction_divider(auto: bool) -> AnyElement {
    let label = if auto {
        text::COMPACTED_AUTO
    } else {
        text::COMPACTED
    };
    div()
        .w_full()
        .py(space::SM)
        .child(Divider::horizontal().label(label))
        .into_any_element()
}
