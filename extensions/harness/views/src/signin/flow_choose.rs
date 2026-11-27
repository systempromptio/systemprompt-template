//! Step 2 of the sign-in flow and its ending: the outcome of a sign-in, the
//! signed-in instances, Finish, the first sync and the fade.

use sp_protocol::ids::AccountId;
use sp_protocol::view::{InstanceSummary, SignInOutcome};

use super::{Direction, Effect, Failure, FirstSync, Flow, Method, Step};
use crate::views::signin::address::InstanceChoice;

impl Flow {
    pub(super) fn ended(&mut self, attempt: u64, outcome: SignInOutcome) -> Vec<Effect> {
        let Some(method) = self.pending.filter(|_| attempt == self.attempt) else {
            return Vec::new();
        };
        self.pending = None;
        let failure = match outcome {
            SignInOutcome::SignedIn => {
                self.signed_in_here = true;
                self.step = Step::Choose;
                self.code_page_opened = false;
                self.choose_error = None;
                let mut effects = vec![Effect::LoadInstances];
                if method == Method::Token {
                    effects.push(Effect::ClearToken);
                }
                return effects;
            },
            SignInOutcome::Cancelled => return Vec::new(),
            SignInOutcome::Denied => Failure::Denied,
            SignInOutcome::Expired => Failure::Expired,
            SignInOutcome::Failed(failure) => Failure::SignIn(failure),
        };
        self.error = Some((method, failure));
        Vec::new()
    }

    pub(super) fn instances_loaded(&mut self, list: Vec<InstanceChoice>) {
        if self.step != Step::Choose {
            return;
        }
        let keep = self
            .selected
            .as_ref()
            .filter(|selected| list.iter().any(|choice| &&choice.account == selected));
        self.selected = keep
            .cloned()
            .or_else(|| list.first().map(|choice| choice.account.clone()));
        self.instances = list;
    }

    pub(super) fn summary_loaded(&mut self, account: &AccountId, summary: InstanceSummary) {
        if let Some(choice) = self
            .instances
            .iter_mut()
            .find(|choice| &choice.account == account)
        {
            choice.summary = Some(summary);
        }
    }

    pub(super) fn select(&mut self, account: AccountId) {
        if self.step == Step::Choose
            && self
                .instances
                .iter()
                .any(|choice| choice.account == account)
        {
            self.selected = Some(account);
            self.choose_error = None;
        }
    }

    pub(super) fn move_selection(&mut self, direction: Direction) {
        if self.step != Step::Choose || self.instances.is_empty() {
            return;
        }
        let last = self.instances.len() - 1;
        let current = self.selected.as_ref().and_then(|selected| {
            self.instances
                .iter()
                .position(|choice| &choice.account == selected)
        });
        let next = match (current, direction) {
            (None, _) => 0,
            (Some(index), Direction::Next) => (index + 1).min(last),
            (Some(index), Direction::Previous) => index.saturating_sub(1),
        };
        self.selected = self
            .instances
            .get(next)
            .map(|choice| choice.account.clone());
        self.choose_error = None;
    }

    pub(super) fn back(&mut self) {
        if self.step != Step::Choose {
            return;
        }
        self.step = Step::Connect;
        self.attempt = self.attempt.wrapping_add(1);
        self.pending = None;
        self.error = None;
        self.choose_error = None;
    }

    pub(super) fn finish(&mut self) -> Vec<Effect> {
        if self.step != Step::Choose {
            return Vec::new();
        }
        let Some(account) = self
            .selected_instance()
            .map(|choice| choice.account.clone())
        else {
            return Vec::new();
        };
        self.step = Step::Finishing;
        self.synced = None;
        self.choose_error = None;
        vec![Effect::Activate(account)]
    }

    pub(super) fn activated(&mut self, activated: bool) -> Vec<Effect> {
        if self.step != Step::Finishing {
            return Vec::new();
        }
        match self.selected.clone() {
            Some(account) if activated => vec![Effect::AwaitFirstSync(account)],
            _ => {
                self.step = Step::Choose;
                self.choose_error = Some(Failure::Activation);
                Vec::new()
            },
        }
    }

    pub(super) fn first_sync(&mut self, result: FirstSync) -> Vec<Effect> {
        if self.step != Step::Finishing {
            return Vec::new();
        }
        self.synced = Some(result);
        self.step = Step::Closing;
        vec![Effect::FadeOut]
    }
}
