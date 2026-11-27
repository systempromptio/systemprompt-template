//! Order-keeping edits to a session's messages and their parts, and the effect
//! each edit had.

use systemprompt_engine::store::{Message, MessageId, MessageWithParts, Part, PartBody, PartId};

/// What folding a change or event into a session model did: nothing visible, a
/// visible change, or a model that must be reloaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelEffect {
    Unchanged,
    Changed,
    Reload,
}

impl ModelEffect {
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

pub(crate) fn upsert_message(
    messages: &mut Vec<MessageWithParts>,
    message: &Message,
) -> ModelEffect {
    match position(messages, message.id()) {
        Ok(index) => {
            let entry = &mut messages[index];
            if entry.message == *message {
                return ModelEffect::Unchanged;
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
    ModelEffect::Changed
}

pub(crate) fn remove_message(messages: &mut Vec<MessageWithParts>, id: &MessageId) -> ModelEffect {
    position(messages, id).map_or(ModelEffect::Unchanged, |index| {
        messages.remove(index);
        ModelEffect::Changed
    })
}

pub(crate) fn upsert_part(messages: &mut [MessageWithParts], part: &Part) -> ModelEffect {
    let Some(entry) = message_mut(messages, &part.message_id) else {
        return ModelEffect::Reload;
    };
    match entry
        .parts
        .binary_search_by(|existing| existing.id.cmp(&part.id))
    {
        Ok(index) => {
            let existing = &mut entry.parts[index];
            if existing.updated_at > part.updated_at || *existing == *part {
                return ModelEffect::Unchanged;
            }
            *existing = part.clone();
        },
        Err(index) => entry.parts.insert(index, part.clone()),
    }
    ModelEffect::Changed
}

pub(crate) fn remove_part(
    messages: &mut [MessageWithParts],
    message_id: &MessageId,
    part_id: &PartId,
) -> ModelEffect {
    let Some(entry) = message_mut(messages, message_id) else {
        return ModelEffect::Unchanged;
    };
    remove_by(&mut entry.parts, |part| &part.id == part_id)
}

pub(crate) fn append_delta(
    messages: &mut [MessageWithParts],
    message_id: &MessageId,
    part_id: &PartId,
    delta: &str,
) -> Option<ModelEffect> {
    let part = message_mut(messages, message_id)?
        .parts
        .iter_mut()
        .find(|part| &part.id == part_id)?;
    match &mut part.body {
        PartBody::Text { text, .. } | PartBody::Reasoning { text, .. } if !delta.is_empty() => {
            text.push_str(delta);
            Some(ModelEffect::Changed)
        },
        _ => Some(ModelEffect::Unchanged),
    }
}

pub(crate) fn remove_by<T>(list: &mut Vec<T>, matches: impl Fn(&T) -> bool) -> ModelEffect {
    let before = list.len();
    list.retain(|item| !matches(item));
    if list.len() == before {
        ModelEffect::Unchanged
    } else {
        ModelEffect::Changed
    }
}
