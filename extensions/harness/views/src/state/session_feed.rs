//! How an open session takes in its load result, store changes and engine
//! events, and settles what they change.

use crate::state::session_model::{ModelEffect, Snapshot};
use crate::state::session_state::SessionState;
use gpui_kit::Context;
use systemprompt_engine::run::{EngineError, EngineEvent};
use systemprompt_engine::store::Change;

impl SessionState {
    pub(super) fn loaded(&mut self, result: Result<Snapshot, EngineError>, cx: &mut Context<Self>) {
        self.loading = false;
        match result {
            Ok(snapshot) => self.model.load(snapshot),
            Err(error) => self.fail(&error, cx),
        }
        let held = std::mem::take(&mut self.held);
        let effect = self.apply(&held.changes, &held.events);
        if effect == ModelEffect::Reload {
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

    pub(super) fn apply(&mut self, changes: &[Change], events: &[EngineEvent]) -> ModelEffect {
        let from_changes = changes
            .iter()
            .fold(ModelEffect::Unchanged, |effect, change| {
                effect.merge(self.model.apply_change(change))
            });
        events.iter().fold(from_changes, |effect, event| {
            effect.merge(self.model.apply_event(event))
        })
    }

    pub(super) fn settle(&mut self, effect: ModelEffect, cx: &mut Context<Self>) {
        match effect {
            ModelEffect::Unchanged => {},
            ModelEffect::Changed => cx.notify(),
            ModelEffect::Reload => self.reload(cx),
        }
    }
}
