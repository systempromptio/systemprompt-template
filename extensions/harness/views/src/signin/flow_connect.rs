//! Step 1 of the sign-in flow: the address as it is typed and checked, and the
//! browser and token sign-ins it starts.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_accounts::ProbeResult;
use sp_protocol::view::SignInFailure;

use super::{
    Check, Effect, Failure, Flow, Method, Problem, RETRY_DELAY, SETTLE_DELAY, SignInRequest, Target,
};
use crate::views::signin::address::{Credential, instance_target};

impl Flow {
    pub(super) fn edited(&mut self, text: String) -> Vec<Effect> {
        if text == self.address {
            return Vec::new();
        }
        self.address = text;
        self.edit = self.edit.wrapping_add(1);
        self.retries = 0;
        self.error = None;
        if self.address.trim().is_empty() {
            self.check = Check::Empty;
            return Vec::new();
        }
        self.check = Check::Typing;
        vec![Effect::SettleAfter {
            edit: self.edit,
            delay: SETTLE_DELAY,
        }]
    }

    pub(super) fn settle_now(&mut self) -> Vec<Effect> {
        if self.check == Check::Invalid(Problem::Unreachable) {
            self.check = Check::Typing;
        }
        self.settle()
    }

    pub(super) fn settle(&mut self) -> Vec<Effect> {
        if self.check != Check::Typing {
            return Vec::new();
        }
        match instance_target(&self.address) {
            None => {
                self.check = Check::Invalid(Problem::Malformed);
                Vec::new()
            },
            Some(address) => {
                self.check = Check::Checking(address.clone());
                vec![Effect::Probe { address }]
            },
        }
    }

    pub(super) fn probed(&mut self, address: &str, result: ProbeResult) -> Vec<Effect> {
        if !matches!(&self.check, Check::Checking(probing) if probing == address) {
            return Vec::new();
        }
        let problem = match result {
            ProbeResult::Instance(found) => {
                let browser = found.browser;
                self.check = Check::Ready(Target {
                    url: found.url,
                    host: found.host,
                    label: found.label,
                    browser,
                });
                if !browser {
                    self.token_open = true;
                }
                return Vec::new();
            },
            ProbeResult::Unreachable if self.retries > 0 => {
                self.retries -= 1;
                self.check = Check::Typing;
                return vec![Effect::SettleAfter {
                    edit: self.edit,
                    delay: RETRY_DELAY,
                }];
            },
            ProbeResult::Unreachable => Problem::Unreachable,
            ProbeResult::Unsupported => Problem::Unsupported,
            ProbeResult::Malformed => Problem::Malformed,
        };
        self.check = Check::Invalid(problem);
        Vec::new()
    }

    pub(super) fn ready_url(&self) -> Option<String> {
        match &self.check {
            Check::Ready(target) => Some(target.url.clone()),
            _ => None,
        }
    }

    pub(super) const fn begin(&mut self, method: Method, url: String) -> SignInRequest {
        self.attempt = self.attempt.wrapping_add(1);
        self.pending = Some(method);
        self.error = None;
        SignInRequest {
            attempt: self.attempt,
            url,
            keep: self.keep,
        }
    }

    pub(super) fn browser(&mut self) -> Vec<Effect> {
        if self.pending.is_some() || !self.secure || self.token_only() {
            return Vec::new();
        }
        let Some(url) = self.ready_url() else {
            return self.settle_now();
        };
        vec![Effect::SignInBrowser(self.begin(Method::Browser, url))]
    }

    pub(super) fn cancel(&mut self) -> Vec<Effect> {
        if self.pending != Some(Method::Browser) {
            return Vec::new();
        }
        self.pending = None;
        self.attempt = self.attempt.wrapping_add(1);
        vec![Effect::CancelBrowser]
    }

    pub(super) fn token(&mut self, credential: Option<Credential>) -> Vec<Effect> {
        if self.pending.is_some() || !self.secure {
            return Vec::new();
        }
        let Some(url) = self.ready_url() else {
            return self.settle_now();
        };
        let Some(credential) = credential else {
            self.error = Some((Method::Token, Failure::MissingToken));
            return Vec::new();
        };
        vec![Effect::SignInToken {
            request: self.begin(Method::Token, url),
            credential,
        }]
    }

    pub(super) fn code_page(&mut self) -> Vec<Effect> {
        let Some(url) = self.ready_url() else {
            return self.settle_now();
        };
        self.clear_error(Method::Token);
        vec![Effect::ShowCodePage { url }]
    }

    pub(super) const fn code_page_answered(&mut self, result: Result<(), SignInFailure>) {
        match result {
            Ok(()) => self.code_page_opened = true,
            Err(failure) => self.error = Some((Method::Token, Failure::SignIn(failure))),
        }
    }
}
