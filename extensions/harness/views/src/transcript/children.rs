//! The progress of subagent sessions a transcript shows on its task rows: how
//! many tool steps each child has taken and what it is doing now, kept current
//! from the store's change feed.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::{BTreeMap, HashMap};

use harness_store::{Change, MessageWithParts, Part, PartBody, PartId, SessionId};

use crate::views::transcript::tool_info::{self, Phase};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChildProgress {
    pub steps: usize,
    pub current: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StepBrief {
    label: String,
    finished: bool,
}

#[derive(Debug, Default)]
pub struct Children {
    sessions: HashMap<SessionId, BTreeMap<PartId, StepBrief>>,
}

impl Children {
    pub fn watch(&mut self, id: &SessionId) -> bool {
        if self.sessions.contains_key(id) {
            return false;
        }
        self.sessions.insert(id.clone(), BTreeMap::new());
        true
    }

    pub(crate) fn load(&mut self, id: &SessionId, messages: &[MessageWithParts]) {
        let steps = self.sessions.entry(id.clone()).or_default();
        for part in messages.iter().flat_map(|entry| entry.parts.iter()) {
            if let Some(brief) = brief(part) {
                steps.insert(part.id.clone(), brief);
            }
        }
    }

    pub fn apply(&mut self, change: &Change) -> bool {
        let Change::PartUpdated(part) = change else {
            return false;
        };
        let Some(steps) = self.sessions.get_mut(&part.session_id) else {
            return false;
        };
        let Some(brief) = brief(part) else {
            return false;
        };
        if steps.get(&part.id) == Some(&brief) {
            return false;
        }
        steps.insert(part.id.clone(), brief);
        true
    }

    pub fn progress(&self, id: &SessionId) -> Option<ChildProgress> {
        let steps = self.sessions.get(id)?;
        let current = steps
            .values()
            .rev()
            .find(|step| !step.finished)
            .or_else(|| steps.values().next_back())
            .map(|step| step.label.clone());
        Some(ChildProgress {
            steps: steps.len(),
            current,
        })
    }
}

fn brief(part: &Part) -> Option<StepBrief> {
    let PartBody::Tool { tool, state, .. } = &part.body else {
        return None;
    };
    let summary = tool_info::summarize(tool, state);
    let label = match &summary.subject {
        Some(subject) => format!("{} {subject}", summary.name),
        None => summary.name.clone(),
    };
    let finished = matches!(summary.phase, Phase::Done | Phase::Failed(_));
    Some(StepBrief { label, finished })
}
