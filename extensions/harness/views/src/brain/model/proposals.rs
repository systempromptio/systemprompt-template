//! The proposals inbox: the proposals the instance's analyses drafted, the
//! decision in flight and how the last one ended. Nothing a proposal describes
//! applies itself; it waits for a person's approval.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_protocol::wire::brain::{BrainProposal, BrainProposalId, ProposalStatus};

use super::actions::{Decision, DecisionOutcome};
use super::failure::BrainFailure;
use super::load::Load;

/// The inbox and what the person is deciding in it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProposalInbox {
    pub proposals: Load<Vec<BrainProposal>>,
    pub pending: Option<(BrainProposalId, Decision)>,
    pub outcome: Option<DecisionOutcome>,
    pub expanded: Option<BrainProposalId>,
}

impl ProposalInbox {
    pub fn open(&self) -> impl Iterator<Item = &BrainProposal> {
        self.with_status(true)
    }

    pub fn decided(&self) -> impl Iterator<Item = &BrainProposal> {
        self.with_status(false)
    }

    fn with_status(&self, open: bool) -> impl Iterator<Item = &BrainProposal> {
        self.proposals
            .ready()
            .into_iter()
            .flatten()
            .filter(move |proposal| (proposal.status == ProposalStatus::Open) == open)
    }

    pub fn open_count(&self) -> usize {
        self.open().count()
    }

    pub fn is_pending(&self, id: &BrainProposalId) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|(pending, _)| pending == id)
    }

    pub fn begin(&mut self, id: &BrainProposalId, decision: Decision) -> bool {
        let open = self.open().any(|proposal| &proposal.id == id);
        if self.pending.is_some() || !open {
            return false;
        }
        self.pending = Some((id.clone(), decision));
        self.outcome = None;
        true
    }

    pub fn finish(&mut self, id: &BrainProposalId, result: Result<(), BrainFailure>) {
        let decision = match &self.pending {
            Some((pending, decision)) if pending == id => *decision,
            _ => return,
        };
        self.pending = None;
        self.outcome = Some(match result {
            Ok(()) => DecisionOutcome::Done(decision),
            Err(failure) => DecisionOutcome::Failed(decision, failure),
        });
    }

    pub fn toggle(&mut self, id: &BrainProposalId) {
        if self.expanded.as_ref() == Some(id) {
            self.expanded = None;
        } else {
            self.expanded = Some(id.clone());
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
