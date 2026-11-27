//! The transcript adornment: under a session's latest reply, how many team
//! facts the instance's gateway recalled into that session's latest model
//! call, read from [`BrainService::memory`]; the fact ids and revisions show
//! on hover; the transcript draws it again whenever the feed's memory count
//! moves. Held as `dyn TranscriptAdornment` on the UI thread.

use std::cell::RefCell;

use gpui_kit::{
    AnyElement, App, AsyncApp, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Task, Window, div,
};
use systemprompt_engine::store::{Message, MessageWithParts};
use systemprompt_harness::members::brain::BrainService;
use systemprompt_harness_shared::components::Badge;
use systemprompt_harness_shared::contract::{OnChange, ShellContext, TranscriptAdornment};
use systemprompt_harness_shared::theme::space;
use systemprompt_harness_shared::{IconName, Tone, tooltip};

use crate::brain::model::LatestReplies;
use crate::brain::strings;

pub(crate) struct RecalledFacts {
    service: BrainService,
    latest: RefCell<LatestReplies>,
}

impl RecalledFacts {
    pub(crate) fn new(service: BrainService) -> Self {
        Self {
            service,
            latest: RefCell::new(LatestReplies::default()),
        }
    }
}

impl TranscriptAdornment for RecalledFacts {
    fn follow(&self, on_change: OnChange, cx: &mut App) -> Option<Task<()>> {
        let mut feed = self.service.subscribe();
        let mut memory = feed.borrow_and_update().memory;
        Some(cx.spawn(async move |cx: &mut AsyncApp| {
            while feed.changed().await.is_ok() {
                let next = feed.borrow_and_update().memory;
                if next != memory {
                    memory = next;
                    cx.update(|cx| on_change(cx));
                }
            }
        }))
    }

    fn adorn(
        &self,
        message: &MessageWithParts,
        _ctx: &ShellContext,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<AnyElement> {
        let Message::Assistant(reply) = &message.message else {
            return None;
        };
        if !self
            .latest
            .borrow_mut()
            .observe(&reply.session_id, &reply.id)
        {
            return None;
        }
        let facts = self.service.memory(&reply.session_id);
        if facts.is_empty() {
            return None;
        }
        let detail = facts
            .iter()
            .map(|fact| format!("{}@{}", fact.id, fact.revision))
            .collect::<Vec<_>>()
            .join("\n");
        let id = SharedString::from(format!("brain-recalled-{}", reply.id));
        Some(
            div()
                .id(id)
                .flex()
                .flex_row()
                .pt(space::XS)
                .child(
                    Badge::new(strings::recalled(facts.len()))
                        .icon(IconName::Sparkle)
                        .tone(Tone::Accent),
                )
                .tooltip(tooltip(detail))
                .into_any_element(),
        )
    }
}
