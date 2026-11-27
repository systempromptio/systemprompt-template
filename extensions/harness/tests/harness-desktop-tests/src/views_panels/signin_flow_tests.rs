use systemprompt_device::accounts::ProbeResult;
use systemprompt_device::view::{SignInFailure, SignInOutcome};
use systemprompt_harness_views::signin::address::Credential;
use systemprompt_harness_views::signin::flow::{
    Check, Effect, Event, Failure, Flow, Method, Problem, RETRY_DELAY, SETTLE_DELAY, Step,
};

use super::signin_flow_fixtures::{ADDRESS, instance, ready};

#[test]
fn the_opening_address_is_checked_at_once_and_typing_waits_for_a_pause() {
    let mut flow = Flow::new(ADDRESS, true);
    assert_eq!(
        flow.start(),
        vec![Effect::Probe {
            address: ADDRESS.to_owned()
        }]
    );
    let effects = flow.handle(Event::Edited("sp.other.com".to_owned()));
    assert_eq!(
        effects,
        vec![Effect::SettleAfter {
            edit: 1,
            delay: SETTLE_DELAY
        }]
    );
    assert_eq!(flow.check, Check::Typing);
    assert_eq!(
        flow.handle(Event::Probed {
            address: ADDRESS.to_owned(),
            result: instance(true)
        }),
        Vec::<Effect>::new()
    );
    assert_eq!(flow.check, Check::Typing);
    flow.handle(Event::Edited("sp.other.co".to_owned()));
    assert_eq!(
        flow.handle(Event::SettleDue { edit: 1 }),
        Vec::<Effect>::new()
    );
    assert_eq!(
        flow.handle(Event::SettleDue { edit: 2 }),
        vec![Effect::Probe {
            address: "https://sp.other.co".to_owned()
        }]
    );
}

#[test]
fn a_malformed_address_is_never_probed_and_an_empty_one_asks_for_input() {
    let mut flow = Flow::new("", true);
    assert_eq!(flow.start(), Vec::<Effect>::new());
    assert_eq!(flow.check, Check::Empty);
    flow.handle(Event::Edited("not an address".to_owned()));
    assert_eq!(flow.handle(Event::SettleNow), Vec::<Effect>::new());
    assert_eq!(flow.check, Check::Invalid(Problem::Malformed));
}

#[test]
fn the_opening_address_is_retried_while_unreachable_but_a_typed_one_is_not() {
    let mut flow = Flow::new(ADDRESS, true);
    flow.start();
    let effects = flow.handle(Event::Probed {
        address: ADDRESS.to_owned(),
        result: ProbeResult::Unreachable,
    });
    assert_eq!(
        effects,
        vec![Effect::SettleAfter {
            edit: 0,
            delay: RETRY_DELAY
        }]
    );
    flow.handle(Event::Edited("sp.typed.com".to_owned()));
    flow.handle(Event::SettleDue { edit: 1 });
    let typed = "https://sp.typed.com".to_owned();
    assert_eq!(
        flow.handle(Event::Probed {
            address: typed,
            result: ProbeResult::Unreachable
        }),
        Vec::<Effect>::new()
    );
    assert_eq!(flow.check, Check::Invalid(Problem::Unreachable));
}

#[test]
fn unsupported_and_token_only_instances_are_told_apart() {
    let mut flow = Flow::new(ADDRESS, true);
    flow.start();
    flow.handle(Event::Probed {
        address: ADDRESS.to_owned(),
        result: ProbeResult::Unsupported,
    });
    assert_eq!(flow.check, Check::Invalid(Problem::Unsupported));
    let token_only = ready(false);
    assert!(token_only.token_only());
    assert!(token_only.token_open);
}

#[test]
fn a_cancelled_browser_sign_in_drops_its_late_answer() {
    let mut flow = ready(true);
    flow.handle(Event::BrowserRequested);
    assert_eq!(flow.pending, Some(Method::Browser));
    assert_eq!(
        flow.handle(Event::CancelRequested),
        vec![Effect::CancelBrowser]
    );
    assert_eq!(
        flow.handle(Event::SignInEnded {
            attempt: 1,
            outcome: SignInOutcome::SignedIn
        }),
        Vec::<Effect>::new()
    );
    assert_eq!(flow.step, Step::Connect);
    assert!(flow.releasable());
}

#[test]
fn failures_show_under_the_control_that_started_them() {
    let mut flow = ready(true);
    flow.handle(Event::BrowserRequested);
    flow.handle(Event::SignInEnded {
        attempt: 1,
        outcome: SignInOutcome::Denied,
    });
    assert_eq!(flow.error, Some((Method::Browser, Failure::Denied)));
    flow.handle(Event::TokenSubmitted(Some(Credential::Pat)));
    assert_eq!(flow.error, None);
    let failed = SignInOutcome::Failed(SignInFailure::Refused);
    flow.handle(Event::SignInEnded {
        attempt: 2,
        outcome: failed,
    });
    assert_eq!(
        flow.error,
        Some((Method::Token, Failure::SignIn(SignInFailure::Refused)))
    );
    flow.handle(Event::CodePageOpened(Err(SignInFailure::Unreachable)));
    assert_eq!(
        flow.error,
        Some((Method::Token, Failure::SignIn(SignInFailure::Unreachable)))
    );
}

#[test]
fn an_empty_token_is_refused_before_any_request() {
    let mut flow = ready(true);
    assert_eq!(
        flow.handle(Event::TokenSubmitted(None)),
        Vec::<Effect>::new()
    );
    assert_eq!(flow.error, Some((Method::Token, Failure::MissingToken)));
    let effects = flow.handle(Event::TokenSubmitted(Some(Credential::Code)));
    assert!(matches!(
        effects.as_slice(),
        [Effect::SignInToken {
            credential: Credential::Code,
            ..
        }]
    ));
}

#[test]
fn without_secure_storage_no_sign_in_starts() {
    let mut flow = ready(true);
    flow.handle(Event::SecureStorage(false));
    assert!(!flow.can_sign_in());
    assert_eq!(flow.handle(Event::BrowserRequested), Vec::<Effect>::new());
    assert_eq!(
        flow.handle(Event::TokenSubmitted(Some(Credential::Pat))),
        Vec::<Effect>::new()
    );
    assert_eq!(flow.pending, None);
}

#[test]
fn a_token_sign_in_clears_the_field_and_keep_follows_the_checkbox() {
    let mut flow = ready(true);
    flow.handle(Event::Keep(false));
    let effects = flow.handle(Event::TokenSubmitted(Some(Credential::Pat)));
    assert!(matches!(effects.as_slice(), [Effect::SignInToken { request, .. }] if !request.keep));
    let ended = flow.handle(Event::SignInEnded {
        attempt: 1,
        outcome: SignInOutcome::SignedIn,
    });
    assert_eq!(ended, vec![Effect::LoadInstances, Effect::ClearToken]);
}
