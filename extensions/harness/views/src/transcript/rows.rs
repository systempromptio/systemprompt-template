//! Which messages of a session become rows of the transcript list, and what
//! changed between two renders, so the list re-measures only the rows whose
//! content moved.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _, Hasher as _};
use std::ops::Range;

use systemprompt_engine::store::{Message, MessageId, MessageWithParts, PartBody};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowKey {
    pub index: usize,
    pub(crate) id: MessageId,
    pub(crate) fingerprint: u64,
}

pub(crate) fn is_shown_part(body: &PartBody) -> bool {
    match body {
        PartBody::Text { text, synthetic } => !synthetic && !text.trim().is_empty(),
        PartBody::Reasoning { text, .. } => !text.trim().is_empty(),
        PartBody::File { .. }
        | PartBody::Tool { .. }
        | PartBody::Compaction { .. }
        | PartBody::Subtask { .. } => true,
        PartBody::StepStart { .. } | PartBody::StepFinish { .. } | PartBody::Patch { .. } => false,
    }
}

pub(crate) fn is_shown(entry: &MessageWithParts, live: bool) -> bool {
    match &entry.message {
        Message::User(user) => {
            !user.system
                || entry
                    .parts
                    .iter()
                    .any(|part| matches!(part.body, PartBody::Compaction { .. }))
        },
        Message::Assistant(reply) => {
            live || reply.error.is_some()
                || entry.parts.iter().any(|part| is_shown_part(&part.body))
        },
    }
}

pub fn row_keys(messages: &[MessageWithParts], busy: bool) -> Vec<RowKey> {
    let last = messages.len().checked_sub(1);
    messages
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let live = busy && Some(index) == last;
            is_shown(entry, live).then(|| RowKey {
                index,
                id: entry.message.id().clone(),
                fingerprint: fingerprint(entry, live),
            })
        })
        .collect()
}

fn fingerprint(entry: &MessageWithParts, live: bool) -> u64 {
    let mut hasher = DefaultHasher::new();
    live.hash(&mut hasher);
    if let Message::Assistant(reply) = &entry.message {
        reply.completed_at.hash(&mut hasher);
        reply.error.is_some().hash(&mut hasher);
        reply
            .finish
            .map(systemprompt_engine::store::FinishReason::as_str)
            .hash(&mut hasher);
    }
    for part in &entry.parts {
        part.id.hash(&mut hasher);
        part.updated_at.hash(&mut hasher);
        part.body.text().map(str::len).hash(&mut hasher);
    }
    hasher.finish()
}

pub fn changed_range(old: &[RowKey], new: &[RowKey]) -> Option<(Range<usize>, usize)> {
    let common = old.len().min(new.len());
    let first = old
        .iter()
        .zip(new)
        .position(|(before, after)| before != after)
        .unwrap_or(common);
    if first == old.len() && first == new.len() {
        return None;
    }
    Some((first..old.len(), new.len() - first))
}

pub(crate) fn following_message(messages: &[MessageWithParts], index: usize) -> Option<MessageId> {
    messages
        .get(index + 1)
        .map(|entry| entry.message.id().clone())
}

pub fn plain_text(entry: &MessageWithParts) -> String {
    entry
        .parts
        .iter()
        .filter_map(|part| match &part.body {
            PartBody::Text {
                text,
                synthetic: false,
            } => Some(text.trim()),
            _ => None,
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}
