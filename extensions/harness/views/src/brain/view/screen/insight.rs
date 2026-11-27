//! The open fact's why-view (the terms of its importance and the reason recall
//! would leave it out) and its revision timeline, newest first.

use gpui_kit::{AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div};
use harness_ui::components::{Badge, Banner, Card, Spinner};
use harness_ui::theme::{space, text, tokens};
use sp_protocol::wire::brain::FactRevision;

use super::BrainScreen;
use super::detail::row;
use crate::model::{FactDetail, Load};
use crate::strings;
use crate::view::tone;

impl BrainScreen {
    pub(super) fn render_why(detail: &FactDetail, cx: &Context<Self>) -> AnyElement {
        let card = Card::new().title(strings::WHY_TITLE);
        let why = match &detail.why {
            Load::Ready(why) => why,
            Load::Idle | Load::Loading => return card.child(Spinner::new()).into_any_element(),
            Load::Failed(failure) => {
                return card
                    .child(muted(strings::failure(*failure), cx))
                    .into_any_element();
            },
        };
        let terms = why.terms;
        let verdict = why.withheld.map_or_else(
            || Banner::info("brain-why-verdict", strings::WHY_RECALLED),
            |reason| Banner::warning("brain-why-verdict", strings::withheld(reason)),
        );
        card.child(row(strings::WHY_BASE, score(terms.base).into(), cx))
            .child(row(strings::WHY_SUBJECT, score(terms.subject).into(), cx))
            .child(row(strings::WHY_WORKFLOW, score(terms.workflow).into(), cx))
            .child(row(strings::WHY_RECENCY, score(terms.recency).into(), cx))
            .child(row(strings::WHY_USAGE, score(terms.usage).into(), cx))
            .child(row(strings::WHY_TOTAL, score(terms.total).into(), cx))
            .child(div().pt(space::SM).child(verdict))
            .into_any_element()
    }

    pub(super) fn render_timeline(detail: &FactDetail, cx: &Context<Self>) -> AnyElement {
        let card = Card::new().title(strings::TIMELINE_TITLE);
        let revisions = match &detail.timeline {
            Load::Ready(revisions) => revisions,
            Load::Idle | Load::Loading => return card.child(Spinner::new()).into_any_element(),
            Load::Failed(failure) => {
                return card
                    .child(muted(strings::failure(*failure), cx))
                    .into_any_element();
            },
        };
        if revisions.is_empty() {
            return card
                .child(muted(strings::NO_TIMELINE, cx))
                .into_any_element();
        }
        card.children(revisions.iter().rev().map(|revision| entry(revision, cx)))
            .into_any_element()
    }
}

fn entry<T>(revision: &FactRevision, cx: &Context<T>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .py(space::SM)
        .border_b_1()
        .border_color(palette.border.subtle)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .text_size(text::SMALL)
                .text_color(palette.text.primary)
                .child(strings::revision_line(revision.revision, revision.change))
                .child(
                    Badge::new(strings::status(revision.status))
                        .tone(tone::status(revision.status)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_color(palette.text.muted)
                        .child(strings::when(&revision.recorded_at)),
                ),
        )
        .child(
            div()
                .text_size(text::SMALL)
                .text_color(palette.text.secondary)
                .child(revision.statement.clone()),
        )
        .into_any_element()
}

fn muted<T>(message: &'static str, cx: &Context<T>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .text_size(text::SMALL)
        .text_color(palette.text.muted)
        .child(message)
        .into_any_element()
}

fn score(value: f64) -> String {
    format!("{value:.2}")
}
