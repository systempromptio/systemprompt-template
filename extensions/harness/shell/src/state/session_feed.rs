//! How an open session takes in its load result, store changes and engine
//! events, and settles what they change.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use crate::state::session_model::{Effect, Snapshot};
use crate::state::session_state::SessionState;
use gpui_kit::Context;
use harness_engine::{EngineError, EngineEvent};
use harness_store::Change;

impl SessionState {
    pub(super) fn loaded(&mut self, result: Result<Snapshot, EngineError>, cx: &mut Context<Self>) {
        self.loading = false;
        match result {
            Ok(snapshot) => self.model.load(snapshot),
            Err(error) => self.fail(&error, cx),
        }
        let held = std::mem::take(&mut self.held);
        let effect = self.apply(&held.changes, &held.events);
        if effect == Effect::Reload {
            self.reload(cx);
        }
        cx.notify();
    }

    pub(super) fn on_changes(&mut self, changes: Vec<Change>, cx: &mut Context<Self>) {
        if self.loading {
            for change in changes {
                if matches!(change, Change::PartDelta { .. }) {
                    self.model.mark_stale();
                } else {
                    self.held.changes.push(change);
                }
            }
            return;
        }
        let effect = self.apply(&changes, &[]);
        self.settle(effect, cx);
    }

    pub(super) fn on_events(&mut self, events: Vec<EngineEvent>, cx: &mut Context<Self>) {
        if self.loading {
            self.held.events.extend(events);
            return;
        }
        let effect = self.apply(&[], &events);
        self.settle(effect, cx);
    }

    pub(super) fn on_lag(&mut self, _missed: u64, cx: &mut Context<Self>) {
        self.reload(cx);
    }

    pub(super) fn apply(&mut self, changes: &[Change], events: &[EngineEvent]) -> Effect {
        let from_changes = changes.iter().fold(Effect::Unchanged, |effect, change| {
            effect.merge(self.model.apply_change(change))
        });
        events.iter().fold(from_changes, |effect, event| {
            effect.merge(self.model.apply_event(event))
        })
    }

    pub(super) fn settle(&mut self, effect: Effect, cx: &mut Context<Self>) {
        match effect {
            Effect::Unchanged => {},
            Effect::Changed => cx.notify(),
            Effect::Reload => self.reload(cx),
        }
    }
}
