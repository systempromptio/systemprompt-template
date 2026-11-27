//! What the Brain screen reads: the fact list a page at a time under its
//! filter, the open fact with its timeline and why-view, and the proposals.

use gpui_kit::{Context, SharedString, Window};
use systemprompt_identifiers::BrainFactId;
use systemprompt_models::brain::FactStatus;

use super::{BrainScreen, bridge};
use crate::brain::model::{BrainFailure, FactDetail, FactFilter, Load};
use crate::brain::strings;

impl BrainScreen {
    pub(super) fn reload_all(&mut self, cx: &mut Context<Self>) {
        self.reload_listing(cx);
        self.reload_detail(cx);
        self.reload_inbox(cx);
    }

    pub(super) fn reload_listing(&mut self, cx: &mut Context<Self>) {
        self.fetch_page(None, cx);
    }

    pub(super) fn load_more(&mut self, cx: &mut Context<Self>) {
        if let Some(after) = self.listing.next.clone() {
            self.fetch_page(Some(after), cx);
        }
    }

    fn fetch_page(&mut self, after: Option<String>, cx: &mut Context<Self>) {
        let generation = self.listing.begin(after.is_some());
        let query = self.listing.filter.query(after);
        let service = self.service.clone();
        let job = async move { service.facts(&query).await };
        let task = bridge::run(
            cx,
            &self.runtime,
            "brain-facts",
            job,
            move |this, result, cx| {
                let applied = match result {
                    Ok(page) => this.listing.apply(generation, page),
                    Err(error) => {
                        tracing::warn!(error = %error, "a brain fact listing failed");
                        this.listing.fail(generation, BrainFailure::of(&error))
                    },
                };
                if applied {
                    cx.notify();
                }
            },
        );
        self.tasks.listing = Some(task);
        cx.notify();
    }

    pub(super) fn apply_filters(&mut self, cx: &mut Context<Self>) {
        let kind = FactFilter::parse_kind(&self.kind_field.read(cx).text(cx));
        let subject = FactFilter::parse_subject(&self.subject_field.read(cx).text(cx));
        let kind_error = kind
            .is_err()
            .then_some(SharedString::new_static(strings::KIND_INVALID));
        let subject_error = subject
            .is_err()
            .then_some(SharedString::new_static(strings::SUBJECT_INVALID));
        self.kind_field
            .update(cx, |field, cx| field.set_error(kind_error, cx));
        self.subject_field
            .update(cx, |field, cx| field.set_error(subject_error, cx));
        match (kind, subject) {
            (Ok(kind), Ok(subject)) => {
                let filter = FactFilter {
                    kind,
                    subject,
                    statuses: self.listing.filter.statuses.clone(),
                };
                self.set_filter(filter, cx);
            },
            (Err(error), _) | (_, Err(error)) => {
                tracing::debug!(error = %error, "a brain filter does not parse");
            },
        }
    }

    pub(super) fn toggle_status(&mut self, status: FactStatus, cx: &mut Context<Self>) {
        let mut filter = self.listing.filter.clone();
        filter.toggle_status(status);
        self.set_filter(filter, cx);
    }

    pub(super) fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for field in [&self.kind_field, &self.subject_field] {
            field.update(cx, |field, cx| {
                field.set_text("", window, cx);
                field.set_error(None, cx);
            });
        }
        self.set_filter(FactFilter::default(), cx);
    }

    fn set_filter(&mut self, filter: FactFilter, cx: &mut Context<Self>) {
        if self.listing.set_filter(filter) {
            self.reload_listing(cx);
        }
    }

    pub(super) fn select_fact(&mut self, id: BrainFactId, cx: &mut Context<Self>) {
        if self.detail.as_ref().is_some_and(|detail| detail.id == id) {
            return;
        }
        self.listing.selected = Some(id.clone());
        self.detail = Some(FactDetail::loading(id));
        self.clear_reason = true;
        self.reload_detail(cx);
    }

    pub(super) fn reload_detail(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.detail.as_ref().map(|detail| detail.id.clone()) else {
            return;
        };
        let service = self.service.clone();
        let target = id.clone();
        let job = async move {
            tokio::join!(
                service.fact(&target),
                service.timeline(&target),
                service.why(&target, None)
            )
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "brain-fact",
            job,
            move |this, (fact, timeline, why), cx| {
                if let Some(detail) = this.detail.as_mut().filter(|detail| detail.id == id) {
                    detail.fact = Load::settle(fact, "fact");
                    detail.timeline = Load::settle(timeline, "timeline");
                    detail.why = Load::settle(why, "why");
                    cx.notify();
                }
            },
        );
        self.tasks.detail = Some(task);
        cx.notify();
    }

    pub(super) fn reload_inbox(&mut self, cx: &mut Context<Self>) {
        if self.inbox.proposals.ready().is_none() {
            self.inbox.proposals = Load::Loading;
        }
        let service = self.service.clone();
        let job = async move { service.proposals(None).await };
        let task = bridge::run(
            cx,
            &self.runtime,
            "brain-proposals",
            job,
            |this, result, cx| {
                this.inbox.proposals = Load::settle(result, "proposals");
                cx.notify();
            },
        );
        self.tasks.inbox = Some(task);
        cx.notify();
    }
}
