//! The open fact: the fact, its revision timeline and its why-view, the reason
//! being written for a dispute or retraction, the action in flight and how the
//! last one ended.

use sp_protocol::wire::brain::{BrainFact, BrainFactId, FactRevision, FactWhy};

use super::actions::{ActionOutcome, FactAction};
use super::failure::BrainFailure;
use super::load::Load;

/// One open fact and what the person is doing with it.
#[derive(Debug, Clone, PartialEq)]
pub struct FactDetail {
    pub id: BrainFactId,
    pub fact: Load<BrainFact>,
    pub timeline: Load<Vec<FactRevision>>,
    pub why: Load<FactWhy>,
    pub drafting: Option<FactAction>,
    pub pending: Option<FactAction>,
    pub outcome: Option<ActionOutcome>,
}

impl FactDetail {
    pub const fn loading(id: BrainFactId) -> Self {
        Self {
            id,
            fact: Load::Loading,
            timeline: Load::Loading,
            why: Load::Loading,
            drafting: None,
            pending: None,
            outcome: None,
        }
    }

    pub fn actions(&self) -> Vec<FactAction> {
        self.fact
            .ready()
            .map(FactAction::available)
            .unwrap_or_default()
    }

    pub const fn draft(&mut self, action: FactAction) -> bool {
        if !action.takes_reason() || self.pending.is_some() {
            return false;
        }
        self.drafting = Some(action);
        self.outcome = None;
        true
    }

    pub const fn cancel_draft(&mut self) {
        self.drafting = None;
    }

    pub fn begin(&mut self, action: FactAction) -> bool {
        if self.pending.is_some() || !self.actions().contains(&action) {
            return false;
        }
        self.pending = Some(action);
        self.drafting = None;
        self.outcome = None;
        true
    }

    pub fn finish(&mut self, action: FactAction, result: Result<(), BrainFailure>) {
        if self.pending != Some(action) {
            return;
        }
        self.pending = None;
        self.outcome = Some(match result {
            Ok(()) => ActionOutcome::Done(action),
            Err(failure) => ActionOutcome::Failed(action, failure),
        });
    }
}
