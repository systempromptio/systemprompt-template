//! What the composer hands to the state: a prompt or a slash command, and the
//! agent and model it goes to.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use harness_engine::{AttachmentInput, CommandInfo, Delivery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    pub text: String,
    pub attachments: Vec<AttachmentInput>,
    pub delivery: Delivery,
}

impl Submission {
    pub fn text(text: impl Into<String>, delivery: Delivery) -> Self {
        Self {
            text: text.into(),
            attachments: Vec::new(),
            delivery,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.attachments.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Selection {
    pub(crate) agent: Option<String>,
    pub(crate) model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCall {
    pub(crate) name: String,
    pub(crate) arguments: String,
    pub(crate) delivery: Delivery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Prompt(Submission),
    Command(CommandCall),
}

pub fn classify(submission: Submission, commands: &[CommandInfo]) -> Intent {
    match slash_command(&submission.text, commands) {
        Some((name, arguments)) if submission.attachments.is_empty() => {
            Intent::Command(CommandCall {
                name,
                arguments,
                delivery: submission.delivery,
            })
        },
        _ => Intent::Prompt(submission),
    }
}

pub fn slash_command(text: &str, commands: &[CommandInfo]) -> Option<(String, String)> {
    let rest = text.trim_start().strip_prefix('/')?;
    let (name, arguments) = match rest.split_once(char::is_whitespace) {
        Some((name, arguments)) => (name, arguments.trim()),
        None => (rest.trim_end(), ""),
    };
    commands
        .iter()
        .any(|command| command.name == name)
        .then(|| (name.to_owned(), arguments.to_owned()))
}
