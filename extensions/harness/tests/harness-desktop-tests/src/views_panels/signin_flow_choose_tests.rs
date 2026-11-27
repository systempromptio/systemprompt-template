use harness_desktop::views::signin::flow::{Direction, Effect, Event, Failure, FirstSync, Step};
use sp_protocol::ids::AccountId;
use sp_protocol::view::InstanceSummary;

use super::signin_flow_fixtures::signed_in;

#[test]
fn step_two_preselects_the_first_instance_and_moves_with_the_keyboard() {
    let mut flow = signed_in();
    assert_eq!(flow.selected, Some(AccountId::new("alpha")));
    flow.handle(Event::Move(Direction::Next));
    flow.handle(Event::Move(Direction::Next));
    assert_eq!(flow.selected, Some(AccountId::new("beta")));
    flow.handle(Event::Move(Direction::Previous));
    assert_eq!(flow.selected, Some(AccountId::new("alpha")));
    flow.handle(Event::Select(AccountId::new("unknown")));
    assert_eq!(flow.selected, Some(AccountId::new("alpha")));
    let summary = InstanceSummary {
        skills: 3,
        agents: 1,
        mcp: 2,
        role: None,
    };
    flow.handle(Event::Summary {
        account: AccountId::new("alpha"),
        summary: summary.clone(),
    });
    assert_eq!(
        flow.selected_instance()
            .and_then(|choice| choice.summary.clone()),
        Some(summary)
    );
}

#[test]
fn going_back_keeps_the_gate_until_finish_has_faded_it_out() {
    let mut flow = signed_in();
    flow.handle(Event::Back);
    assert_eq!(flow.step, Step::Connect);
    assert!(!flow.releasable());
    let mut flow = signed_in();
    assert_eq!(
        flow.handle(Event::FinishRequested),
        vec![Effect::Activate(AccountId::new("alpha"))]
    );
    assert_eq!(flow.step, Step::Finishing);
    assert_eq!(
        flow.handle(Event::Activated(true)),
        vec![Effect::AwaitFirstSync(AccountId::new("alpha"))]
    );
    assert_eq!(
        flow.handle(Event::FirstSync(FirstSync::Synced { skills: 4 })),
        vec![Effect::FadeOut]
    );
    assert_eq!(flow.step, Step::Closing);
    assert!(!flow.releasable());
    flow.handle(Event::Faded);
    assert_eq!(flow.step, Step::Closed);
    assert!(flow.releasable());
}

#[test]
fn a_failed_activation_returns_to_the_choice_with_its_message() {
    let mut flow = signed_in();
    flow.handle(Event::FinishRequested);
    assert_eq!(flow.handle(Event::Activated(false)), Vec::<Effect>::new());
    assert_eq!(flow.step, Step::Choose);
    assert_eq!(flow.choose_error, Some(Failure::Activation));
}
