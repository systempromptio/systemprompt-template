//! The Brain screen: the facts tab (filters, the paged list, the open fact with
//! its timeline, why-view and the person's actions) and the proposals inbox.
//!
//! It renders from the [`BrainService`] it was built with and follows its feed:
//! a change of status or enablement reloads or clears everything, a facts
//! change reloads the list and the open fact, a proposals change reloads the
//! inbox. Unless the instance admits the extension and has confirmed its
//! brain, the screen shows why instead of any data.

mod acts;
mod detail;
mod facts;
mod insight;
mod loads;
mod proposals;
mod render;

use gpui_kit::{AppContext as _, Context, Entity, Task, Window};
use systemprompt_harness::extension::Enablement;
use systemprompt_harness::members::brain::{BrainFeed, BrainService};
use systemprompt_harness_shared::components::TextField;
use systemprompt_harness_shared::contract::ShellContext;
use tokio::runtime::Handle;

use super::bridge;
use crate::brain::model::{Availability, FactDetail, FactListing, ProposalInbox};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Facts,
    Proposals,
}

#[derive(Default)]
struct Tasks {
    listing: Option<Task<()>>,
    detail: Option<Task<()>>,
    inbox: Option<Task<()>>,
    actions: Vec<Task<()>>,
}

pub(crate) struct BrainScreen {
    service: BrainService,
    runtime: Handle,
    enablement: Enablement,
    tab: Tab,
    ready: bool,
    seen: BrainFeed,
    listing: FactListing,
    detail: Option<FactDetail>,
    inbox: ProposalInbox,
    kind_field: Entity<TextField>,
    subject_field: Entity<TextField>,
    reason_field: Entity<TextField>,
    clear_reason: bool,
    tasks: Tasks,
    _feeds: [Task<()>; 2],
}

impl BrainScreen {
    pub(crate) fn new(
        ctx: &ShellContext,
        service: BrainService,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let kind_field = Self::filter_field(crate::brain::strings::KIND_PLACEHOLDER, window, cx);
        let subject_field =
            Self::filter_field(crate::brain::strings::SUBJECT_PLACEHOLDER, window, cx);
        let weak = cx.weak_entity();
        let reason_field = cx.new(|cx| {
            TextField::new(window, cx).on_submit(move |_, _, cx| {
                if weak.update(cx, Self::submit_reason).is_err() {
                    tracing::trace!("reason submitted after the Brain closed");
                }
            })
        });
        let feeds = [
            bridge::follow(cx, service.subscribe(), Self::follow_feed),
            bridge::follow(cx, ctx.enablement.subscribe(), |this, _, cx| {
                this.follow_availability(cx);
            }),
        ];
        let mut screen = Self {
            seen: service.feed(),
            service,
            runtime: ctx.runtime.clone(),
            enablement: ctx.enablement.clone(),
            tab: Tab::default(),
            ready: false,
            listing: FactListing::default(),
            detail: None,
            inbox: ProposalInbox::default(),
            kind_field,
            subject_field,
            reason_field,
            clear_reason: false,
            tasks: Tasks::default(),
            _feeds: feeds,
        };
        screen.follow_availability(cx);
        screen
    }

    fn filter_field(
        placeholder: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextField> {
        let weak = cx.weak_entity();
        cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(placeholder)
                .on_submit(move |_, _, cx| {
                    if weak.update(cx, Self::apply_filters).is_err() {
                        tracing::trace!("filter submitted after the Brain closed");
                    }
                })
        })
    }

    fn availability(&self) -> Availability {
        Availability::of(self.enablement.current().as_ref(), self.service.status())
    }

    fn follow_availability(&mut self, cx: &mut Context<Self>) -> bool {
        let ready = self.availability().is_ready();
        let reloaded = ready && !self.ready;
        if reloaded {
            self.reload_all(cx);
        } else if !ready && self.ready {
            self.clear();
        }
        self.ready = ready;
        cx.notify();
        reloaded
    }

    fn follow_feed(&mut self, feed: BrainFeed, cx: &mut Context<Self>) {
        let seen = std::mem::replace(&mut self.seen, feed);
        if self.follow_availability(cx) || !self.ready {
            return;
        }
        if feed.facts != seen.facts {
            self.reload_listing(cx);
            self.reload_detail(cx);
        }
        if feed.proposals != seen.proposals {
            self.reload_inbox(cx);
        }
    }

    fn clear(&mut self) {
        self.listing.clear();
        self.detail = None;
        self.inbox.clear();
        self.tasks = Tasks::default();
        self.clear_reason = true;
    }

    fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.actions.retain(|task| !task.is_ready());
        self.tasks.actions.push(task);
    }
}
