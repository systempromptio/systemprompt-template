//! Fact actions, the open fact, availability, failures and latest replies.

use systemprompt_harness::extension::{EnabledExtensions, extension_id};
use systemprompt_harness::members::brain::{BRAIN_EXTENSION, BrainError, BrainStatus};
use systemprompt_harness_views::brain::model::{
    ActionOutcome, Availability, BrainFailure, FactAction, FactDetail, LatestReplies, Load,
};
use systemprompt_harness_views::brain::strings;
use systemprompt_identifiers::BrainFactId;
use systemprompt_identifiers::engine::{MessageId, SessionId};
use systemprompt_models::brain::FactStatus;

use crate::TestResult;
use crate::fixtures::{FACT_A, fact};

#[test]
fn actions_follow_the_fact_status_and_pin() -> TestResult {
    let candidate = fact(FACT_A, FactStatus::Candidate, false)?;
    assert_eq!(
        FactAction::available(&candidate),
        [
            FactAction::Confirm,
            FactAction::Dispute,
            FactAction::Retract,
            FactAction::Pin
        ]
    );
    let confirmed = fact(FACT_A, FactStatus::Confirmed, true)?;
    assert_eq!(
        FactAction::available(&confirmed),
        [FactAction::Dispute, FactAction::Retract, FactAction::Unpin]
    );
    let disputed = fact(FACT_A, FactStatus::Disputed, false)?;
    assert!(!FactAction::available(&disputed).contains(&FactAction::Dispute));
    assert_eq!(
        FactAction::available(&fact(FACT_A, FactStatus::Retired, false)?),
        []
    );
    assert_eq!(
        FactAction::available(&fact(FACT_A, FactStatus::Superseded, true)?),
        [FactAction::Unpin]
    );
    Ok(())
}

#[test]
fn a_dispute_needs_a_reason_and_a_retraction_does_not() {
    assert!(FactAction::Dispute.takes_reason());
    assert!(FactAction::Retract.takes_reason());
    assert!(!FactAction::Confirm.takes_reason());
    assert!(!FactAction::Dispute.accepts("   "));
    assert!(FactAction::Dispute.accepts("wrong client"));
    assert!(FactAction::Retract.accepts(""));
}

fn open_detail(status: FactStatus) -> TestResult<FactDetail> {
    let id = BrainFactId::try_new(FACT_A)?;
    let mut detail = FactDetail::loading(id);
    detail.fact = Load::Ready(fact(FACT_A, status, false)?);
    Ok(detail)
}

#[test]
fn the_open_fact_runs_one_available_action_at_a_time() -> TestResult {
    let mut detail = open_detail(FactStatus::Candidate)?;
    assert!(!detail.begin(FactAction::Unpin));
    assert!(detail.begin(FactAction::Confirm));
    assert!(!detail.begin(FactAction::Pin));
    detail.finish(FactAction::Pin, Ok(()));
    assert_eq!(detail.pending, Some(FactAction::Confirm));
    detail.finish(FactAction::Confirm, Err(BrainFailure::Forbidden));
    assert_eq!(detail.pending, None);
    assert_eq!(
        detail.outcome,
        Some(ActionOutcome::Failed(
            FactAction::Confirm,
            BrainFailure::Forbidden
        ))
    );
    Ok(())
}

#[test]
fn drafting_a_reason_only_applies_to_actions_that_take_one() -> TestResult {
    let mut detail = open_detail(FactStatus::Confirmed)?;
    assert!(!detail.draft(FactAction::Pin));
    assert!(detail.draft(FactAction::Dispute));
    assert_eq!(detail.drafting, Some(FactAction::Dispute));
    assert!(detail.begin(FactAction::Dispute));
    assert_eq!(detail.drafting, None);
    detail.finish(FactAction::Dispute, Ok(()));
    assert_eq!(
        detail.outcome,
        Some(ActionOutcome::Done(FactAction::Dispute))
    );
    Ok(())
}

#[test]
fn a_loading_fact_offers_no_action() -> TestResult {
    let detail = FactDetail::loading(BrainFactId::try_new(FACT_A)?);
    assert_eq!(detail.actions(), []);
    Ok(())
}

#[test]
fn availability_is_withheld_unless_the_instance_enables_the_brain() {
    assert_eq!(
        Availability::of(None, BrainStatus::Enabled),
        Availability::Withheld
    );
    let other: EnabledExtensions = std::iter::once(extension_id!("drive")).collect();
    assert_eq!(
        Availability::of(Some(&other), BrainStatus::Enabled),
        Availability::Withheld
    );
    let enabled: EnabledExtensions = std::iter::once(BRAIN_EXTENSION).collect();
    assert_eq!(
        Availability::of(Some(&enabled), BrainStatus::Detached),
        Availability::Detached
    );
    assert_eq!(
        Availability::of(Some(&enabled), BrainStatus::Checking),
        Availability::Checking
    );
    assert_eq!(
        Availability::of(Some(&enabled), BrainStatus::Disabled),
        Availability::Disabled
    );
    assert!(Availability::of(Some(&enabled), BrainStatus::Enabled).is_ready());
}

#[test]
fn errors_reduce_to_what_the_person_can_act_on() {
    assert_eq!(
        BrainFailure::of(&BrainError::Disabled),
        BrainFailure::Disabled
    );
    assert_eq!(
        BrainFailure::of(&BrainError::NoInstance),
        BrainFailure::NoInstance
    );
    let forbidden = BrainError::Forbidden {
        endpoint: "POST /api/v1/brain/facts/{id}/confirm",
        error_key: None,
    };
    assert_eq!(BrainFailure::of(&forbidden), BrainFailure::Forbidden);
    let invalid = BrainError::InvalidInput { what: "reason" };
    assert_eq!(BrainFailure::of(&invalid), BrainFailure::Invalid);
    assert!(BrainFailure::Unreachable.is_retryable());
    assert!(!BrainFailure::Forbidden.is_retryable());
}

#[test]
fn only_a_sessions_latest_reply_is_adorned() {
    let mut latest = LatestReplies::default();
    let session = SessionId::mint(&crate::fixtures::IDS);
    let other = SessionId::mint(&crate::fixtures::IDS);
    let older = MessageId::mint(&crate::fixtures::IDS);
    let newer = MessageId::mint(&crate::fixtures::IDS);
    assert!(latest.observe(&session, &older));
    assert!(latest.observe(&session, &newer));
    assert!(!latest.observe(&session, &older));
    assert!(latest.observe(&session, &newer));
    assert!(latest.observe(&other, &older));
}

#[test]
fn counted_strings_read_naturally() {
    assert_eq!(strings::recalled(1), "1 team fact recalled");
    assert_eq!(strings::recalled(3), "3 team facts recalled");
    assert_eq!(strings::open_proposals(2), "Proposals (2)");
    assert!(
        FactStatus::ALL
            .into_iter()
            .all(|status| !strings::status(status).is_empty())
    );
}
