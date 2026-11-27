//! What a person does with a fact or a proposal from the shell. These are the
//! person's own decisions: none of them is ever a tool the model can call.
//! Which of them the account may take is the instance's decision; a refusal
//! comes back as a failure.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_protocol::wire::brain::{BrainFact, FactStatus};

use super::failure::BrainFailure;

/// An action on one fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactAction {
    Confirm,
    Dispute,
    Retract,
    Pin,
    Unpin,
}

impl FactAction {
    pub const ALL: [Self; 5] = [
        Self::Confirm,
        Self::Dispute,
        Self::Retract,
        Self::Pin,
        Self::Unpin,
    ];

    pub fn available(fact: &BrainFact) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|action| action.applies_to(fact.status, fact.pinned))
            .collect()
    }

    pub const fn applies_to(self, status: FactStatus, pinned: bool) -> bool {
        if status.is_terminal() {
            return matches!(self, Self::Unpin) && pinned;
        }
        match self {
            Self::Confirm => !matches!(status, FactStatus::Confirmed),
            Self::Dispute => !matches!(status, FactStatus::Disputed),
            Self::Retract => true,
            Self::Pin => !pinned,
            Self::Unpin => pinned,
        }
    }

    pub const fn takes_reason(self) -> bool {
        matches!(self, Self::Dispute | Self::Retract)
    }

    pub fn accepts(self, reason: &str) -> bool {
        !matches!(self, Self::Dispute) || !reason.trim().is_empty()
    }
}

/// How the last action on the open fact ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    Done(FactAction),
    Failed(FactAction, BrainFailure),
}

/// A decision on a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decision {
    Approve,
    Reject,
}

/// How the last decision in the inbox ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionOutcome {
    Done(Decision),
    Failed(Decision, BrainFailure),
}
