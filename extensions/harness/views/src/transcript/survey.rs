//! What one look at the session model tells the transcript: its revision, the
//! rows to show, the run status, the subagent sessions to follow and the parts
//! still alive.

use std::collections::HashSet;

use harness_engine::RunStatus;
use harness_store::{PartBody, PartId, SessionId};

use crate::state::session_model::SessionModel;
use crate::views::transcript::rows::{self, RowKey};
use crate::views::transcript::tool_info;

pub(super) struct Survey {
    pub(super) revision: u64,
    pub(super) rows: Vec<RowKey>,
    pub(super) status: RunStatus,
    pub(super) children: Vec<SessionId>,
    pub(super) live_parts: HashSet<PartId>,
}

pub(super) fn survey(model: &SessionModel) -> Survey {
    let messages = model.visible_messages();
    let mut children = Vec::new();
    let mut live_parts = HashSet::new();
    for part in messages.iter().flat_map(|entry| entry.parts.iter()) {
        live_parts.insert(part.id.clone());
        match &part.body {
            PartBody::Tool { tool, state, .. }
                if tool_info::ToolKind::of(tool) == tool_info::ToolKind::Task =>
            {
                children.extend(tool_info::child_session(state));
            },
            PartBody::Subtask {
                child_session: Some(child),
                ..
            } => children.push(child.clone()),
            _ => {},
        }
    }
    Survey {
        revision: model.revision,
        rows: rows::row_keys(messages, model.is_busy()),
        status: model.status.clone(),
        children,
        live_parts,
    }
}
