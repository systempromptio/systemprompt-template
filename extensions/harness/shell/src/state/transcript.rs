//! Order-keeping edits to a session's messages and their parts, and the effect
//! each edit had.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use harness_store::{Message, MessageId, MessageWithParts, Part, PartBody, PartId};

/// What folding a change or event into a session model did: nothing visible, a
/// visible change, or a model that must be reloaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Unchanged,
    Changed,
    Reload,
}

impl Effect {
    pub const fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Reload, _) | (_, Self::Reload) => Self::Reload,
            (Self::Changed, _) | (_, Self::Changed) => Self::Changed,
            _ => Self::Unchanged,
        }
    }
}

fn position(messages: &[MessageWithParts], id: &MessageId) -> Result<usize, usize> {
    messages.binary_search_by(|entry| entry.message.id().cmp(id))
}

pub(crate) fn message_mut<'a>(
    messages: &'a mut [MessageWithParts],
    id: &MessageId,
) -> Option<&'a mut MessageWithParts> {
    let index = position(messages, id).ok()?;
    messages.get_mut(index)
}

pub(crate) fn upsert_message(messages: &mut Vec<MessageWithParts>, message: &Message) -> Effect {
    match position(messages, message.id()) {
        Ok(index) => {
            let entry = &mut messages[index];
            if entry.message == *message {
                return Effect::Unchanged;
            }
            entry.message = message.clone();
        },
        Err(index) => messages.insert(
            index,
            MessageWithParts {
                message: message.clone(),
                parts: Vec::new(),
            },
        ),
    }
    Effect::Changed
}

pub(crate) fn remove_message(messages: &mut Vec<MessageWithParts>, id: &MessageId) -> Effect {
    position(messages, id).map_or(Effect::Unchanged, |index| {
        messages.remove(index);
        Effect::Changed
    })
}

pub(crate) fn upsert_part(messages: &mut [MessageWithParts], part: &Part) -> Effect {
    let Some(entry) = message_mut(messages, &part.message_id) else {
        return Effect::Reload;
    };
    match entry
        .parts
        .binary_search_by(|existing| existing.id.cmp(&part.id))
    {
        Ok(index) => {
            let existing = &mut entry.parts[index];
            if existing.updated_at > part.updated_at || *existing == *part {
                return Effect::Unchanged;
            }
            *existing = part.clone();
        },
        Err(index) => entry.parts.insert(index, part.clone()),
    }
    Effect::Changed
}

pub(crate) fn remove_part(
    messages: &mut [MessageWithParts],
    message_id: &MessageId,
    part_id: &PartId,
) -> Effect {
    let Some(entry) = message_mut(messages, message_id) else {
        return Effect::Unchanged;
    };
    remove_by(&mut entry.parts, |part| &part.id == part_id)
}

pub(crate) fn append_delta(
    messages: &mut [MessageWithParts],
    message_id: &MessageId,
    part_id: &PartId,
    delta: &str,
) -> Option<Effect> {
    let part = message_mut(messages, message_id)?
        .parts
        .iter_mut()
        .find(|part| &part.id == part_id)?;
    match &mut part.body {
        PartBody::Text { text, .. } | PartBody::Reasoning { text, .. } if !delta.is_empty() => {
            text.push_str(delta);
            Some(Effect::Changed)
        },
        _ => Some(Effect::Unchanged),
    }
}

pub(crate) fn remove_by<T>(list: &mut Vec<T>, matches: impl Fn(&T) -> bool) -> Effect {
    let before = list.len();
    list.retain(|item| !matches(item));
    if list.len() == before {
        Effect::Unchanged
    } else {
        Effect::Changed
    }
}
