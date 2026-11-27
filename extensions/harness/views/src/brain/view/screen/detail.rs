//! The open fact: its statement and badges, the person's actions with the
//! reason a dispute or retraction asks for, how the last action ended, and
//! the fact's details.

use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div,
};
use harness_ui::components::{Badge, Banner, Button, ButtonKind, Card, EmptyState};
use harness_ui::theme::{space, text, tokens};
use harness_ui::{IconName, Tone};
use serde_json::Value;
use sp_protocol::wire::brain::BrainFact;

use super::BrainScreen;
use crate::model::{ActionOutcome, FactAction, FactDetail, Load};
use crate::strings;
use crate::view::tone;

impl BrainScreen {
    pub(super) fn render_detail(&self, cx: &Context<Self>) -> AnyElement {
        let Some(detail) = &self.detail else {
            return EmptyState::new(strings::SELECT_FACT_TITLE)
                .icon(IconName::Sparkle)
                .description(strings::SELECT_FACT_BODY)
                .into_any_element();
        };
        let fact = match &detail.fact {
            Load::Ready(fact) => fact,
            Load::Idle | Load::Loading => return Self::waiting(),
            Load::Failed(failure) => return Self::failed(*failure, cx),
        };
        div()
            .id("brain-detail")
            .flex()
            .flex_col()
            .gap(space::LG)
            .size_full()
            .p(space::XL)
            .overflow_y_scroll()
            .child(header(fact, cx))
            .child(self.render_actions(detail, cx))
            .children(detail.outcome.map(outcome))
            .child(details(fact, cx))
            .child(Self::render_why(detail, cx))
            .child(Self::render_timeline(detail, cx))
            .into_any_element()
    }

    fn render_actions(&self, detail: &FactDetail, cx: &Context<Self>) -> AnyElement {
        let buttons = detail
            .actions()
            .into_iter()
            .enumerate()
            .map(|(index, action)| {
                Button::new(("brain-action", index))
                    .label(strings::action(action))
                    .small()
                    .kind(action_kind(action))
                    .selected(detail.drafting == Some(action))
                    .loading(detail.pending == Some(action))
                    .disabled(detail.pending.is_some())
                    .on_click(cx.listener(move |this, _, _, cx| this.start_action(action, cx)))
            });
        let draft = detail.drafting.map(|action| self.render_reason(action, cx));
        div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(space::XS)
                    .children(buttons),
            )
            .children(draft)
            .into_any_element()
    }

    fn render_reason(&self, action: FactAction, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let prompt = match action {
            FactAction::Dispute => strings::DISPUTE_PROMPT,
            _ => strings::RETRACT_PROMPT,
        };
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(
                div()
                    .text_size(text::SMALL)
                    .text_color(palette.text.secondary)
                    .child(prompt),
            )
            .child(self.reason_field.clone())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(space::XS)
                    .child(
                        Button::new("brain-reason-submit")
                            .label(strings::SUBMIT)
                            .small()
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| this.submit_reason(cx))),
                    )
                    .child(
                        Button::new("brain-reason-cancel")
                            .label(strings::CANCEL)
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| this.cancel_reason(cx))),
                    ),
            )
            .into_any_element()
    }
}

const fn action_kind(action: FactAction) -> ButtonKind {
    match action {
        FactAction::Confirm => ButtonKind::Primary,
        FactAction::Retract => ButtonKind::Danger,
        FactAction::Dispute | FactAction::Pin | FactAction::Unpin => ButtonKind::Secondary,
    }
}

fn outcome(outcome: ActionOutcome) -> Banner {
    match outcome {
        ActionOutcome::Done(action) => Banner::new(
            "brain-action-outcome",
            Tone::Success,
            strings::action_done(action),
        ),
        ActionOutcome::Failed(_, failure) => {
            Banner::error("brain-action-outcome", strings::failure(failure))
        },
    }
}

fn header<T>(fact: &BrainFact, cx: &Context<T>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_col()
        .gap(space::SM)
        .child(
            div()
                .text_size(text::TITLE)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text.primary)
                .child(fact.statement.clone()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(space::XS)
                .child(Badge::new(strings::status(fact.status)).tone(tone::status(fact.status)))
                .child(Badge::new(strings::scope(fact.scope.level())))
                .children(
                    fact.pinned
                        .then(|| Badge::new(strings::PINNED).tone(Tone::Accent)),
                ),
        )
        .into_any_element()
}

fn details<T>(fact: &BrainFact, cx: &Context<T>) -> AnyElement {
    let confirmed: SharedString = fact.confirmed_at.as_ref().map_or_else(
        || SharedString::new_static(strings::NOT_CONFIRMED),
        |at| strings::when(at).into(),
    );
    let mut rows = vec![
        row(
            strings::DETAIL_KIND,
            fact.kind.as_str().to_owned().into(),
            cx,
        ),
        row(strings::DETAIL_SUBJECT, fact.subject.to_string().into(), cx),
        row(strings::DETAIL_SCOPE, scope_text(fact).into(), cx),
        row(
            strings::DETAIL_OBSERVED,
            strings::when(&fact.observed_at).into(),
            cx,
        ),
        row(strings::DETAIL_CONFIRMED, confirmed, cx),
        row(
            strings::DETAIL_REVISION,
            fact.revision.to_string().into(),
            cx,
        ),
    ];
    rows.extend(
        fact.fields
            .iter()
            .map(|(name, value)| row_owned(name.clone().into(), field_text(value).into(), cx)),
    );
    Card::new().children(rows).into_any_element()
}

fn scope_text(fact: &BrainFact) -> String {
    let level = strings::scope(fact.scope.level());
    fact.scope
        .key()
        .map_or_else(|| level.to_owned(), |key| strings::fact_line(level, key))
}

// JSON: a fact field is typed by the fact's kind declaration, which only the
// instance holds.
fn field_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

pub(super) fn row<T>(label: &'static str, value: SharedString, cx: &Context<T>) -> AnyElement {
    row_owned(SharedString::new_static(label), value, cx)
}

fn row_owned<T>(label: SharedString, value: SharedString, cx: &Context<T>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .gap(space::LG)
        .py(space::XXS)
        .text_size(text::SMALL)
        .child(
            div()
                .w(gpui_kit::px(140.0))
                .flex_shrink_0()
                .text_color(palette.text.muted)
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(palette.text.primary)
                .child(value),
        )
        .into_any_element()
}
