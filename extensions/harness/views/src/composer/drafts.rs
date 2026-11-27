//! What the composer holds for each session while the user is elsewhere: the
//! unsent text and attachments, keyed by session, with the new-session draft
//! under no session.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use harness_engine::AttachmentInput;
use harness_store::SessionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub label: String,
    pub input: AttachmentInput,
}

impl Attachment {
    pub fn file(path: &Path) -> Self {
        let label = path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        Self {
            label,
            input: AttachmentInput::Path(path.to_path_buf()),
        }
    }

    pub fn mention(relative: &str) -> Self {
        Self {
            label: relative.to_owned(),
            input: AttachmentInput::Path(PathBuf::from(relative)),
        }
    }
}

pub fn add(list: &mut Vec<Attachment>, added: impl IntoIterator<Item = Attachment>) -> bool {
    let before = list.len();
    for attachment in added {
        if !list.iter().any(|known| known.input == attachment.input) {
            list.push(attachment);
        }
    }
    list.len() != before
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    pub attachments: Vec<Attachment>,
}

impl Draft {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.attachments.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct Drafts {
    entries: HashMap<Option<SessionId>, Draft>,
}

impl Drafts {
    pub fn keep(&mut self, session: Option<SessionId>, draft: Draft) {
        if draft.is_empty() {
            self.entries.remove(&session);
        } else {
            self.entries.insert(session, draft);
        }
    }

    pub fn take(&mut self, session: Option<&SessionId>) -> Draft {
        self.entries.remove(&session.cloned()).unwrap_or_default()
    }
}
