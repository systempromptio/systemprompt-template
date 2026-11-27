//! The sign-in flow as plain data.
//!
//! It holds what the gate shows and which work it asks for, driven by events
//! from the view and from the background work. Nothing here touches the UI
//! toolkit or the network, so the flow is tested as data.

use std::time::Duration;

use systemprompt_device::accounts::ProbeResult;
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::{InstanceSummary, SignInFailure, SignInOutcome};

use super::address::{Credential, InstanceChoice};

#[path = "flow_choose.rs"]
mod choose;
#[path = "flow_connect.rs"]
mod connect;

pub const SETTLE_DELAY: Duration = Duration::from_millis(500);
pub const RETRY_DELAY: Duration = Duration::from_millis(750);
pub(crate) const OPENING_RETRIES: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Connect,
    Choose,
    Finishing,
    Closing,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub(crate) url: String,
    pub(crate) host: String,
    pub(crate) label: String,
    pub(crate) browser: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    Malformed,
    Unreachable,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Empty,
    Typing,
    Checking(String),
    Ready(Target),
    Invalid(Problem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Browser,
    Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Denied,
    Expired,
    SignIn(SignInFailure),
    MissingToken,
    Activation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstSync {
    Synced { skills: u32 },
    Failed,
    TimedOut,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Previous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Edited(String),
    SettleDue {
        edit: u64,
    },
    SettleNow,
    Probed {
        address: String,
        result: ProbeResult,
    },
    SecureStorage(bool),
    Keep(bool),
    ToggleToken,
    TokenEdited,
    BrowserRequested,
    CancelRequested,
    TokenSubmitted(Option<Credential>),
    CodePageRequested,
    CodePageOpened(Result<(), SignInFailure>),
    SignInEnded {
        attempt: u64,
        outcome: SignInOutcome,
    },
    Instances(Vec<InstanceChoice>),
    Summary {
        account: AccountId,
        summary: InstanceSummary,
    },
    Select(AccountId),
    Move(Direction),
    Back,
    FinishRequested,
    Activated(bool),
    FirstSync(FirstSync),
    Faded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInRequest {
    pub attempt: u64,
    pub(crate) url: String,
    pub keep: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    SettleAfter {
        edit: u64,
        delay: Duration,
    },
    Probe {
        address: String,
    },
    SignInBrowser(SignInRequest),
    CancelBrowser,
    SignInToken {
        request: SignInRequest,
        credential: Credential,
    },
    ShowCodePage {
        url: String,
    },
    ClearToken,
    LoadInstances,
    Activate(AccountId),
    AwaitFirstSync(AccountId),
    FadeOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub step: Step,
    pub(crate) address: String,
    pub check: Check,
    pub(crate) secure: bool,
    pub(crate) keep: bool,
    pub token_open: bool,
    pub(crate) code_page_opened: bool,
    pub(crate) signed_in_here: bool,
    pub pending: Option<Method>,
    pub error: Option<(Method, Failure)>,
    pub(crate) instances: Vec<InstanceChoice>,
    pub selected: Option<AccountId>,
    pub choose_error: Option<Failure>,
    pub(crate) synced: Option<FirstSync>,
    edit: u64,
    attempt: u64,
    retries: u8,
}

impl Flow {
    pub fn new(address: &str, secure: bool) -> Self {
        let empty = address.trim().is_empty();
        Self {
            step: Step::Connect,
            address: address.to_owned(),
            check: if empty { Check::Empty } else { Check::Typing },
            secure,
            keep: true,
            token_open: false,
            code_page_opened: false,
            signed_in_here: false,
            pending: None,
            error: None,
            instances: Vec::new(),
            selected: None,
            choose_error: None,
            synced: None,
            edit: 0,
            attempt: 0,
            retries: if empty { 0 } else { OPENING_RETRIES },
        }
    }

    pub fn start(&mut self) -> Vec<Effect> {
        self.settle()
    }

    pub fn releasable(&self) -> bool {
        self.step == Step::Closed
            || (self.step == Step::Connect && self.pending.is_none() && !self.signed_in_here)
    }

    pub const fn token_only(&self) -> bool {
        matches!(&self.check, Check::Ready(target) if !target.browser)
    }

    pub const fn can_sign_in(&self) -> bool {
        self.secure && self.pending.is_none() && matches!(self.check, Check::Ready(_))
    }

    pub fn selected_instance(&self) -> Option<&InstanceChoice> {
        let selected = self.selected.as_ref()?;
        self.instances
            .iter()
            .find(|choice| &choice.account == selected)
    }

    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::Edited(text) => self.edited(text),
            Event::SettleDue { edit } if edit == self.edit => self.settle(),
            Event::SettleDue { .. } => Vec::new(),
            Event::SettleNow => self.settle_now(),
            Event::Probed { address, result } => self.probed(&address, result),
            Event::SecureStorage(secure) => self.set(|flow| flow.secure = secure),
            Event::Keep(keep) => self.set(|flow| flow.keep = keep),
            Event::ToggleToken => self.set(|flow| flow.token_open = !flow.token_open),
            Event::TokenEdited => self.set(|flow| flow.clear_error(Method::Token)),
            Event::BrowserRequested => self.browser(),
            Event::CancelRequested => self.cancel(),
            Event::TokenSubmitted(credential) => self.token(credential),
            Event::CodePageRequested => self.code_page(),
            Event::CodePageOpened(result) => self.set(|flow| flow.code_page_answered(result)),
            Event::SignInEnded { attempt, outcome } => self.ended(attempt, outcome),
            Event::Instances(list) => self.set(|flow| flow.instances_loaded(list)),
            Event::Summary { account, summary } => {
                self.set(|flow| flow.summary_loaded(&account, summary))
            },
            Event::Select(account) => self.set(|flow| flow.select(account)),
            Event::Move(direction) => self.set(|flow| flow.move_selection(direction)),
            Event::Back => self.set(Self::back),
            Event::FinishRequested => self.finish(),
            Event::Activated(activated) => self.activated(activated),
            Event::FirstSync(result) => self.first_sync(result),
            Event::Faded => self.set(|flow| {
                if flow.step == Step::Closing {
                    flow.step = Step::Closed;
                }
            }),
        }
    }

    fn set(&mut self, change: impl FnOnce(&mut Self)) -> Vec<Effect> {
        change(self);
        Vec::new()
    }

    fn clear_error(&mut self, method: Method) {
        if self.error.is_some_and(|(failed, _)| failed == method) {
            self.error = None;
        }
    }
}
