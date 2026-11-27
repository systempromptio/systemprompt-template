//! The work the sign-in flow asks for, run off the UI thread through
//! `crate::app::runtime`, each result fed back to the flow as an event. A task
//! the gate replaces or drops stops delivering; a browser sign-in is ended
//! through its canceller.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use systemprompt_device::accounts::SignInCancel;
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_harness::sync::SyncEngine;
use systemprompt_harness_shared::motion;

use super::address::{self, Credential};
use super::flow::{Effect, Event, FirstSync, SignInRequest, Step};
use super::{SignInGate, UiRequest};
use crate::app::runtime;

const FIRST_SYNC_LIMIT: Duration = Duration::from_secs(20);

#[derive(Default)]
pub(super) struct Work {
    settle: Option<Task<()>>,
    probe: Option<Task<()>>,
    sign_in: Option<Task<()>>,
    canceller: Option<SignInCancel>,
    code_page: Option<Task<()>>,
    summaries: Vec<Task<()>>,
    activation: Option<Task<()>>,
    first_sync: Option<Task<()>>,
    fade: Option<Task<()>>,
}

impl SignInGate {
    pub(super) fn apply(&mut self, event: Event, cx: &mut Context<Self>) {
        let (step, token_open) = (self.flow.step, self.flow.token_open);
        let effects = self.flow.handle(event);
        self.run_all(effects, cx);
        if self.flow.step != step {
            self.requests.push(UiRequest::FocusStep);
        }
        if self.flow.token_open && !token_open {
            self.requests.push(UiRequest::FocusToken);
        }
        let locked = self.flow.pending.is_some();
        if locked != self.locked {
            self.locked = locked;
            self.requests.push(UiRequest::Lock(locked));
        }
        if self.flow.step == Step::Closed && self.engaged {
            tracing::debug!("sign-in gate released");
            self.engaged = false;
        }
        cx.notify();
    }

    pub(super) fn run_all(&mut self, effects: Vec<Effect>, cx: &mut Context<Self>) {
        for effect in effects {
            self.run(effect, cx);
        }
    }

    fn run(&mut self, effect: Effect, cx: &mut Context<Self>) {
        match effect {
            Effect::SettleAfter { edit, delay } => {
                self.work.settle = Some(after(delay, Event::SettleDue { edit }, cx));
            },
            Effect::Probe { address } => self.probe(address, cx),
            Effect::SignInBrowser(request) => self.sign_in_browser(request, cx),
            Effect::CancelBrowser => self.cancel_browser(),
            Effect::SignInToken {
                request,
                credential,
            } => self.sign_in_token(request, credential, cx),
            Effect::ShowCodePage { url } => self.show_code_page(url, cx),
            Effect::ClearToken => self.requests.push(UiRequest::ClearToken),
            Effect::LoadInstances => self.load_instances(cx),
            Effect::Activate(account) => self.activate(account, cx),
            Effect::AwaitFirstSync(account) => self.await_first_sync(account, cx),
            Effect::FadeOut => {
                let delay = if motion::reduced(cx) {
                    Duration::ZERO
                } else {
                    motion::NORMAL
                };
                self.work.fade = Some(after(delay, Event::Faded, cx));
            },
        }
    }

    fn probe(&mut self, address: String, cx: &Context<Self>) {
        let accounts = self.services.accounts().clone();
        let job = async move {
            let result = accounts.probe(&address).await;
            (address, result)
        };
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            |this, (address, result), cx| {
                this.apply(Event::Probed { address, result }, cx);
            },
        );
        self.work.probe = Some(task);
    }

    fn sign_in_browser(&mut self, request: SignInRequest, cx: &Context<Self>) {
        let SignInRequest { attempt, url, keep } = request;
        let sign_in = self.services.accounts().sign_in_browser(&url, keep);
        self.work.canceller = Some(sign_in.canceller());
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            sign_in.outcome(),
            move |this, outcome, cx| {
                this.work.canceller = None;
                this.apply(Event::SignInEnded { attempt, outcome }, cx);
            },
        );
        self.work.sign_in = Some(task);
    }

    fn cancel_browser(&mut self) {
        self.work.sign_in = None;
        match self.work.canceller.take() {
            Some(canceller) => canceller.cancel(),
            None => self.services.accounts().cancel_sign_in(),
        }
    }

    fn sign_in_token(
        &mut self,
        request: SignInRequest,
        credential: Credential,
        cx: &Context<Self>,
    ) {
        let SignInRequest { attempt, url, keep } = request;
        let secret = self.token.read(cx).text(cx).to_string();
        let accounts = self.services.accounts().clone();
        let job = async move {
            match credential {
                Credential::Code => accounts.sign_in_code(&url, secret, keep).await,
                Credential::Pat => accounts.sign_in_pat(&url, secret, keep).await,
            }
        };
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            move |this, outcome, cx| {
                this.apply(Event::SignInEnded { attempt, outcome }, cx);
            },
        );
        self.work.sign_in = Some(task);
    }

    fn show_code_page(&mut self, url: String, cx: &Context<Self>) {
        let accounts = self.services.accounts().clone();
        let job = async move { accounts.open_code_page(&url).await };
        let task = runtime::spawn(cx, self.services.runtime(), job, |this, result, cx| {
            this.apply(Event::CodePageOpened(result), cx);
        });
        self.work.code_page = Some(task);
    }

    fn load_instances(&mut self, cx: &mut Context<Self>) {
        let accounts = self.services.accounts().clone();
        let mut list = address::choices(&accounts.snapshot());
        for choice in &mut list {
            choice.summary = accounts.last_summary(&choice.account);
        }
        let summaries: Vec<Task<()>> = list
            .iter()
            .filter(|choice| choice.summary.is_none())
            .map(|choice| self.summary(choice.account.clone(), cx))
            .collect();
        self.apply(Event::Instances(list), cx);
        self.work.summaries = summaries;
    }

    fn summary(&self, account: AccountId, cx: &Context<Self>) -> Task<()> {
        let accounts = self.services.accounts().clone();
        let id = account.clone();
        let job = async move { accounts.summary(&id).await };
        runtime::spawn(
            cx,
            self.services.runtime(),
            job,
            move |this, result, cx| match result {
                Ok(summary) => this.apply(Event::Summary { account, summary }, cx),
                Err(error) => {
                    tracing::info!(account = %account, error = %error, "instance summary not read");
                },
            },
        )
    }

    fn activate(&mut self, account: AccountId, cx: &Context<Self>) {
        let accounts = self.services.accounts().clone();
        let task = runtime::blocking(
            cx,
            self.services.runtime(),
            move || accounts.activate(&account),
            |this, result, cx| {
                if let Err(error) = &result {
                    tracing::warn!(error = %error, "the chosen instance was not activated");
                }
                this.apply(Event::Activated(result.is_ok()), cx);
            },
        );
        self.work.activation = Some(task);
    }

    fn await_first_sync(&mut self, account: AccountId, cx: &mut Context<Self>) {
        let Some(engine) = self.services.handles().get::<SyncEngine>() else {
            self.apply(Event::FirstSync(FirstSync::Unavailable), cx);
            return;
        };
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            first_sync(engine, account),
            |this, result, cx| {
                this.apply(Event::FirstSync(result), cx);
            },
        );
        self.work.first_sync = Some(task);
    }
}

fn after(delay: Duration, event: Event, cx: &Context<SignInGate>) -> Task<()> {
    cx.spawn(
        async move |this: WeakEntity<SignInGate>, cx: &mut AsyncApp| {
            cx.background_executor().timer(delay).await;
            if this.update(cx, |this, cx| this.apply(event, cx)).is_err() {
                tracing::trace!("sign-in timer fired after the gate closed");
            }
        },
    )
}

async fn first_sync(engine: Arc<SyncEngine>, account: AccountId) -> FirstSync {
    let mut statuses = engine.subscribe();
    let wait = async move {
        loop {
            let current = address::settled(&statuses.borrow_and_update(), &account);
            if let Some(result) = current {
                return result;
            }
            if statuses.changed().await.is_err() {
                return FirstSync::Unavailable;
            }
        }
    };
    match tokio::time::timeout(FIRST_SYNC_LIMIT, wait).await {
        Ok(result) => result,
        Err(_elapsed) => FirstSync::TimedOut,
    }
}
