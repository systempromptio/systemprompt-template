//! The proposals inbox: the open proposals with their approve and reject
//! actions, then the decided ones; each proposal's body shows on request.

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div,
};
use harness_ui::components::{Badge, Banner, Button, Card, EmptyState, SectionHeader};
use harness_ui::theme::{space, text, tokens};
use harness_ui::{CodeBlock, IconName, Tone};
use sp_protocol::wire::brain::{BrainProposal, ProposalStatus};

use super::BrainScreen;
use crate::model::{Decision, DecisionOutcome, Load};
use crate::strings;
use crate::view::tone;

impl BrainScreen {
    pub(super) fn render_proposals(&self, cx: &Context<Self>) -> AnyElement {
        let proposals = match &self.inbox.proposals {
            Load::Ready(proposals) => proposals,
            Load::Idle | Load::Loading => return Self::waiting(),
            Load::Failed(failure) => return Self::failed(*failure, cx),
        };
        if proposals.is_empty() {
            return EmptyState::new(strings::NO_PROPOSALS_TITLE)
                .icon(IconName::ListTodo)
                .description(strings::NO_PROPOSALS_BODY)
                .into_any_element();
        }
        let decided: Vec<AnyElement> = self
            .inbox
            .decided()
            .enumerate()
            .map(|(index, proposal)| self.proposal(("brain-decided", index), proposal, cx))
            .collect();
        div()
            .id("brain-proposals")
            .flex()
            .flex_col()
            .gap(space::MD)
            .size_full()
            .p(space::XL)
            .overflow_y_scroll()
            .children(self.inbox.outcome.map(outcome))
            .child(SectionHeader::new(strings::PROPOSALS_OPEN).count(self.inbox.open_count()))
            .children(
                self.inbox
                    .open()
                    .enumerate()
                    .map(|(index, proposal)| self.proposal(("brain-open", index), proposal, cx)),
            )
            .children(
                (!decided.is_empty())
                    .then(|| SectionHeader::new(strings::PROPOSALS_DECIDED).count(decided.len())),
            )
            .children(decided)
            .into_any_element()
    }

    fn proposal(
        &self,
        key: (&'static str, usize),
        proposal: &BrainProposal,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let (prefix, index) = key;
        let expanded = self.inbox.expanded.as_ref() == Some(&proposal.id);
        let pending = self
            .inbox
            .pending
            .as_ref()
            .filter(|(id, _)| id == &proposal.id)
            .map(|(_, decision)| *decision);
        let toggled = proposal.id.clone();
        let mut actions = vec![
            Button::new((prefix, index * 3))
                .label(if expanded {
                    strings::HIDE_BODY
                } else {
                    strings::SHOW_BODY
                })
                .small()
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| this.toggle_proposal(&toggled, cx)))
                .into_any_element(),
        ];
        if proposal.status == ProposalStatus::Open {
            for (offset, decision) in [(1, Decision::Approve), (2, Decision::Reject)] {
                let target = proposal.id.clone();
                let button = Button::new((prefix, index * 3 + offset))
                    .label(strings::decision(decision))
                    .small()
                    .loading(pending == Some(decision))
                    .disabled(self.inbox.pending.is_some());
                let button = match decision {
                    Decision::Approve => button.primary(),
                    Decision::Reject => button.danger(),
                };
                actions.push(
                    button
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.decide(target.clone(), decision, cx);
                        }))
                        .into_any_element(),
                );
            }
        }
        let body =
            expanded.then(|| CodeBlock::new((prefix, index), body_text(proposal)).language("json"));
        Card::new()
            .title(proposal.summary.clone())
            .trailing(
                Badge::new(strings::proposal_status(proposal.status))
                    .tone(tone::proposal(proposal.status)),
            )
            .child(
                div()
                    .text_size(text::SMALL)
                    .text_color(palette.text.secondary)
                    .child(strings::proposal_line(
                        strings::proposal_kind(proposal.kind),
                        &proposal.analysis,
                        &strings::when(&proposal.created_at),
                    )),
            )
            .children(body)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(space::XS)
                    .pt(space::SM)
                    .children(actions),
            )
            .into_any_element()
    }
}

fn outcome(outcome: DecisionOutcome) -> Banner {
    match outcome {
        DecisionOutcome::Done(decision) => Banner::new(
            "brain-decision-outcome",
            Tone::Success,
            strings::decision_done(decision),
        ),
        DecisionOutcome::Failed(_, failure) => {
            Banner::error("brain-decision-outcome", strings::failure(failure))
        },
    }
}

fn body_text(proposal: &BrainProposal) -> String {
    match serde_json::to_string_pretty(&proposal.body) {
        Ok(text) => text,
        Err(error) => {
            tracing::debug!(error = %error, "a proposal body does not pretty-print; shown compact");
            proposal.body.to_string()
        },
    }
}
