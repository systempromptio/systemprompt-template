use harness_desktop::views::signin::address::InstanceChoice;
use harness_desktop::views::signin::flow::{Effect, Event, Flow};
use sp_accounts::{ProbeResult, ProbedInstance};
use sp_protocol::ids::AccountId;
use sp_protocol::view::SignInOutcome;

pub(crate) const ADDRESS: &str = "https://sp.example.com";

pub(crate) fn instance(browser: bool) -> ProbeResult {
    ProbeResult::Instance(ProbedInstance {
        url: ADDRESS.to_owned(),
        host: "sp.example.com".to_owned(),
        label: "Example".to_owned(),
        version: None,
        modes: Vec::new(),
        browser,
        sign_in_hint: None,
    })
}

pub(crate) fn ready(browser: bool) -> Flow {
    let mut flow = Flow::new(ADDRESS, true);
    assert_eq!(
        flow.start(),
        vec![Effect::Probe {
            address: ADDRESS.to_owned()
        }]
    );
    assert_eq!(
        flow.handle(Event::Probed {
            address: ADDRESS.to_owned(),
            result: instance(browser)
        }),
        Vec::<Effect>::new()
    );
    flow
}

pub(crate) fn choice(id: &str) -> InstanceChoice {
    InstanceChoice {
        account: AccountId::new(id),
        label: id.to_owned(),
        host: format!("{id}.example.com"),
        user: None,
        fingerprint: None,
        trusted: false,
        summary: None,
    }
}

pub(crate) fn signed_in() -> Flow {
    let mut flow = ready(true);
    let effects = flow.handle(Event::BrowserRequested);
    assert!(
        matches!(effects.as_slice(), [Effect::SignInBrowser(request)] if request.attempt == 1 && request.keep)
    );
    assert_eq!(
        flow.handle(Event::SignInEnded {
            attempt: 1,
            outcome: SignInOutcome::SignedIn
        }),
        vec![Effect::LoadInstances]
    );
    flow.handle(Event::Instances(vec![choice("alpha"), choice("beta")]));
    flow
}
