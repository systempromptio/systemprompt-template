//! What the person does from the Brain screen: confirm, dispute, retract, pin
//! or unpin the open fact, and approve or reject a proposal. A change the
//! instance accepts bumps the brain's feed, which reloads what it touched.

use gpui_kit::{Context, SharedString};
use sp_brain::{BrainError, BrainService};
use sp_protocol::wire::brain::{BrainFactId, BrainProposalId};

use super::{BrainScreen, bridge};
use crate::model::{BrainFailure, Decision, FactAction};
use crate::strings;

impl BrainScreen {
    pub(super) fn start_action(&mut self, action: FactAction, cx: &mut Context<Self>) {
        if !action.takes_reason() {
            self.run_action(action, String::new(), cx);
            return;
        }
        if self
            .detail
            .as_mut()
            .is_some_and(|detail| detail.draft(action))
        {
            self.clear_reason = true;
        }
        cx.notify();
    }

    pub(super) fn submit_reason(&mut self, cx: &mut Context<Self>) {
        let Some(action) = self.detail.as_ref().and_then(|detail| detail.drafting) else {
            return;
        };
        let reason = self.reason_field.read(cx).text(cx).trim().to_owned();
        if !action.accepts(&reason) {
            self.reason_field.update(cx, |field, cx| {
                field.set_error(Some(SharedString::new_static(strings::REASON_REQUIRED)), cx);
            });
            return;
        }
        self.clear_reason = true;
        self.run_action(action, reason, cx);
    }

    pub(super) fn cancel_reason(&mut self, cx: &mut Context<Self>) {
        if let Some(detail) = self.detail.as_mut() {
            detail.cancel_draft();
        }
        self.clear_reason = true;
        cx.notify();
    }

    fn run_action(&mut self, action: FactAction, reason: String, cx: &mut Context<Self>) {
        let Some(detail) = self.detail.as_mut() else {
            return;
        };
        if !detail.begin(action) {
            return;
        }
        let id = detail.id.clone();
        let service = self.service.clone();
        let target = id.clone();
        let job = async move { perform(&service, &target, action, &reason).await };
        let task = bridge::run(
            cx,
            &self.runtime,
            "brain-fact-action",
            job,
            move |this, result, cx| {
                let result = result.map_err(|error| {
                    tracing::warn!(action = ?action, error = %error, "a brain fact action failed");
                    BrainFailure::of(&error)
                });
                if let Some(detail) = this.detail.as_mut().filter(|detail| detail.id == id) {
                    detail.finish(action, result);
                }
                cx.notify();
            },
        );
        self.keep(task);
        cx.notify();
    }

    pub(super) fn decide(
        &mut self,
        id: BrainProposalId,
        decision: Decision,
        cx: &mut Context<Self>,
    ) {
        if !self.inbox.begin(&id, decision) {
            return;
        }
        let service = self.service.clone();
        let target = id.clone();
        let job = async move {
            match decision {
                Decision::Approve => service.approve(&target).await,
                Decision::Reject => service.reject(&target).await,
            }
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "brain-proposal-decision",
            job,
            move |this, result, cx| {
                let result = result.map_err(|error| {
                tracing::warn!(decision = ?decision, error = %error, "a brain proposal decision failed");
                BrainFailure::of(&error)
            });
                this.inbox.finish(&id, result);
                cx.notify();
            },
        );
        self.keep(task);
        cx.notify();
    }

    pub(super) fn toggle_proposal(&mut self, id: &BrainProposalId, cx: &mut Context<Self>) {
        self.inbox.toggle(id);
        cx.notify();
    }
}

async fn perform(
    service: &BrainService,
    id: &BrainFactId,
    action: FactAction,
    reason: &str,
) -> Result<(), BrainError> {
    match action {
        FactAction::Confirm => service.confirm(id).await.map(|_revision| ()),
        FactAction::Dispute => service.dispute(id, reason).await.map(|_dispute| ()),
        FactAction::Retract => service.retract(id, reason).await.map(|_revision| ()),
        FactAction::Pin => service.pin(id).await.map(|_revision| ()),
        FactAction::Unpin => service.unpin(id).await.map(|_revision| ()),
    }
}
