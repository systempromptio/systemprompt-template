//! The question dock: one to four questions with their options, chosen one or
//! several at a time, a typed answer of the user's own for each, and Submit or
//! Dismiss.

use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _,
    div,
};
use systemprompt_engine::run::{Question, QuestionAsk, QuestionId};
use systemprompt_harness_shared::components::{Badge, Button, ListRow};
use systemprompt_harness_shared::theme::{size, space, text as type_scale, tokens};
use systemprompt_harness_shared::{Icon, IconName};

use crate::docks::DocksView;
use crate::docks::permission::{dock_card, dock_title, with_key};
use crate::strings::docks as text;

impl DocksView {
    pub(super) fn render_question(
        &self,
        ask: &QuestionAsk,
        waiting: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let title = if ask.questions.len() == 1 {
            text::QUESTION_TITLE
        } else {
            text::QUESTIONS_TITLE
        };
        let questions: Vec<AnyElement> = ask
            .questions
            .iter()
            .enumerate()
            .map(|(index, question)| self.render_one(index, question, cx))
            .collect();
        let ready = self
            .typed_answers(cx)
            .is_some_and(|(draft_ready, _)| draft_ready);
        let ask_for_submit = ask.clone();
        let dismiss = ask.id.clone();
        let actions = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .child(with_key(
                Button::new("dock-question-submit")
                    .label(text::SUBMIT)
                    .primary()
                    .small()
                    .disabled(!ready)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.submit_question(&ask_for_submit, cx);
                    })),
                text::KEY_ENTER,
            ))
            .child(with_key(
                Button::new("dock-question-dismiss")
                    .label(text::DISMISS)
                    .ghost()
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.dismiss_question(&dismiss, cx)),
                    ),
                text::KEY_REJECT,
            ));
        dock_card(cx)
            .child(dock_title(title, waiting, cx))
            .children(questions)
            .child(actions)
            .into_any_element()
    }

    fn render_one(&self, index: usize, question: &Question, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let mut column = div().flex().flex_col().gap(space::XS);
        let mut heading = div().flex().flex_row().items_center().gap(space::SM);
        if !question.header.trim().is_empty() {
            heading = heading.child(Badge::new(question.header.clone()));
        }
        heading = heading.child(
            div()
                .flex_1()
                .font_weight(FontWeight::MEDIUM)
                .text_color(palette.text.primary)
                .child(SharedString::from(question.question.clone())),
        );
        if question.multiple {
            heading = heading.child(
                div()
                    .text_size(type_scale::CAPTION)
                    .text_color(palette.text.muted)
                    .child(text::PICK_SEVERAL),
            );
        }
        column = column.child(heading);
        for (option, choice) in question.options.iter().enumerate() {
            let chosen = self
                .form
                .as_ref()
                .is_some_and(|form| form.draft.is_chosen(index, option));
            let icon = match (chosen, question.multiple) {
                (true, true) => IconName::Check,
                (true, false) => IconName::CircleCheck,
                (false, _) => IconName::Minus,
            };
            let color = if chosen {
                palette.text.accent
            } else {
                palette.text.muted
            };
            let mut row = ListRow::new(("dock-option", index * 16 + option), choice.label.clone())
                .leading(Icon::new(icon).size(size::ICON_SM).color(color))
                .selected(chosen)
                .on_click(cx.listener(move |this, _, _, cx| this.toggle_option(index, option, cx)));
            if !choice.description.trim().is_empty() {
                row = row.subtitle(choice.description.clone());
            }
            column = column.child(row);
        }
        if let Some(field) = self.form.as_ref().and_then(|form| form.typed.get(index)) {
            column = column.child(field.clone());
        }
        column.into_any_element()
    }

    fn toggle_option(&mut self, question: usize, option: usize, cx: &mut Context<Self>) {
        if let Some(form) = self.form.as_mut() {
            form.draft.toggle(question, option);
            cx.notify();
        }
    }

    fn typed_answers(&self, cx: &Context<Self>) -> Option<(bool, Vec<String>)> {
        let form = self.form.as_ref()?;
        let typed: Vec<String> = form
            .typed
            .iter()
            .map(|field| field.read(cx).text(cx).to_string())
            .collect();
        Some((form.draft.has_answer(&typed), typed))
    }

    pub(super) fn submit_question(&self, ask: &QuestionAsk, cx: &mut Context<Self>) {
        let Some((true, typed)) = self.typed_answers(cx) else {
            return;
        };
        let Some(form) = self.form.as_ref() else {
            return;
        };
        let answers = form.draft.answers(&ask.questions, &typed);
        let id = ask.id.clone();
        self.session.update(cx, |state, cx| {
            state.answer_question(&id, Some(answers), cx);
        });
    }

    pub(super) fn dismiss_question(&self, id: &QuestionId, cx: &mut Context<Self>) {
        self.session
            .update(cx, |state, cx| state.answer_question(id, None, cx));
    }
}
