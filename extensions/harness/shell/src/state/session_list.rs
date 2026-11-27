//! The sessions of the open project as the sidebar lists them.
//!
//! Roots come newest first, subagent children under their parent, grouped by
//! day and filtered by a search. Pure data and functions; the sidebar renders
//! the rows it produces.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use harness_store::{Session, SessionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayBucket {
    Today,
    Yesterday,
    PreviousWeek,
    PreviousMonth,
    Older,
}

pub fn bucket(today: NaiveDate, day: NaiveDate) -> DayBucket {
    match (today - day).num_days() {
        i64::MIN..=0 => DayBucket::Today,
        1 => DayBucket::Yesterday,
        2..=7 => DayBucket::PreviousWeek,
        8..=30 => DayBucket::PreviousMonth,
        _ => DayBucket::Older,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRow {
    Header(DayBucket),
    Session {
        id: SessionId,
        depth: u8,
        has_children: bool,
        expanded: bool,
    },
}

/// The open project's sessions: roots newest first, each parent's subagent
/// children under it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionList {
    roots: Vec<Session>,
    children: HashMap<SessionId, Vec<Session>>,
}

impl SessionList {
    pub fn replace(&mut self, roots: Vec<Session>, children: Vec<Session>) {
        self.roots.clear();
        self.children.clear();
        for session in roots.into_iter().chain(children) {
            self.upsert(session);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.roots.clear();
        self.children.clear();
    }

    pub const fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub fn roots(&self) -> &[Session] {
        &self.roots
    }

    pub fn children(&self, parent: &SessionId) -> &[Session] {
        self.children
            .get(parent)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn get(&self, id: &SessionId) -> Option<&Session> {
        self.roots
            .iter()
            .chain(self.children.values().flatten())
            .find(|session| &session.id == id)
    }

    pub fn contains(&self, id: &SessionId) -> bool {
        self.get(id).is_some()
    }

    pub fn upsert(&mut self, session: Session) -> bool {
        if session.is_archived() {
            return self.remove(&session.id);
        }
        let list = match &session.parent_id {
            Some(parent) => self.children.entry(parent.clone()).or_default(),
            None => &mut self.roots,
        };
        if let Some(existing) = list.iter_mut().find(|existing| existing.id == session.id) {
            if *existing == session {
                return false;
            }
            *existing = session;
        } else {
            list.push(session);
        }
        list.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        true
    }

    pub fn remove(&mut self, id: &SessionId) -> bool {
        let before = self.roots.len();
        self.roots.retain(|session| &session.id != id);
        let mut changed = self.roots.len() != before;
        changed |= self.children.remove(id).is_some();
        for list in self.children.values_mut() {
            let count = list.len();
            list.retain(|session| &session.id != id);
            changed |= list.len() != count;
        }
        changed
    }

    pub fn rows(
        &self,
        query: &str,
        today: NaiveDate,
        expanded: &HashSet<SessionId>,
        day_of: impl Fn(i64) -> NaiveDate,
    ) -> Vec<SessionRow> {
        let needle = query.trim().to_lowercase();
        let matches =
            |session: &Session| needle.is_empty() || session.title.to_lowercase().contains(&needle);
        let mut rows = Vec::new();
        let mut current = None;
        for root in &self.roots {
            let children = self.children(&root.id);
            let child_hits: Vec<&Session> =
                children.iter().filter(|child| matches(child)).collect();
            if !matches(root) && child_hits.is_empty() {
                continue;
            }
            let group = bucket(today, day_of(root.updated_at));
            if current != Some(group) {
                rows.push(SessionRow::Header(group));
                current = Some(group);
            }
            let open =
                expanded.contains(&root.id) || (!needle.is_empty() && !child_hits.is_empty());
            rows.push(SessionRow::Session {
                id: root.id.clone(),
                depth: 0,
                has_children: !children.is_empty(),
                expanded: open,
            });
            if open {
                let shown: Vec<&Session> = if needle.is_empty() {
                    children.iter().collect()
                } else {
                    child_hits
                };
                rows.extend(shown.into_iter().map(|child| SessionRow::Session {
                    id: child.id.clone(),
                    depth: 1,
                    has_children: false,
                    expanded: false,
                }));
            }
        }
        rows
    }
}
