//! The proposals inbox: open and decided proposals and one decision at a time.

use systemprompt_harness_views::brain::model::{
    BrainFailure, Decision, DecisionOutcome, Load, ProposalInbox,
};
use systemprompt_identifiers::BrainProposalId;

use crate::fixtures::{PROPOSAL_DONE, PROPOSAL_OPEN, proposal};

fn inbox() -> ProposalInbox {
    ProposalInbox {
        proposals: Load::Ready(vec![
            proposal(PROPOSAL_OPEN, "open"),
            proposal(PROPOSAL_DONE, "approved"),
        ]),
        ..ProposalInbox::default()
    }
}

fn id(value: &str) -> BrainProposalId {
    BrainProposalId::try_new(value).expect("a proposal id")
}

#[test]
fn open_and_decided_proposals_are_told_apart() {
    let inbox = inbox();
    assert_eq!(inbox.open_count(), 1);
    assert_eq!(
        inbox.open().next().map(|p| p.id.clone()),
        Some(id(PROPOSAL_OPEN))
    );
    assert_eq!(
        inbox.decided().next().map(|p| p.id.clone()),
        Some(id(PROPOSAL_DONE))
    );
    assert_eq!(ProposalInbox::default().open_count(), 0);
}

#[test]
fn only_an_open_proposal_is_decided_and_one_at_a_time() {
    let mut inbox = inbox();
    assert!(!inbox.begin(&id(PROPOSAL_DONE), Decision::Approve));
    assert!(inbox.begin(&id(PROPOSAL_OPEN), Decision::Reject));
    assert!(inbox.is_pending(&id(PROPOSAL_OPEN)));
    assert!(!inbox.begin(&id(PROPOSAL_OPEN), Decision::Approve));

    inbox.finish(&id(PROPOSAL_DONE), Ok(()));
    assert!(inbox.is_pending(&id(PROPOSAL_OPEN)));
    inbox.finish(&id(PROPOSAL_OPEN), Err(BrainFailure::Conflict));
    assert!(!inbox.is_pending(&id(PROPOSAL_OPEN)));
    assert_eq!(
        inbox.outcome,
        Some(DecisionOutcome::Failed(
            Decision::Reject,
            BrainFailure::Conflict
        ))
    );

    assert!(inbox.begin(&id(PROPOSAL_OPEN), Decision::Approve));
    inbox.finish(&id(PROPOSAL_OPEN), Ok(()));
    assert_eq!(
        inbox.outcome,
        Some(DecisionOutcome::Done(Decision::Approve))
    );
}

#[test]
fn a_proposal_body_toggles_open_and_closed() {
    let mut inbox = inbox();
    inbox.toggle(&id(PROPOSAL_OPEN));
    assert_eq!(inbox.expanded, Some(id(PROPOSAL_OPEN)));
    inbox.toggle(&id(PROPOSAL_OPEN));
    assert_eq!(inbox.expanded, None);
}
