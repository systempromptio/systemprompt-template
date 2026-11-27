//! The English text of the composer: placeholders, send and stop, delivery
//! hints, attachments, the command and file lists.

pub(crate) const PLACEHOLDER: &str = "Message — / for commands, @ to mention a file";
pub(crate) const PLACEHOLDER_BUSY: &str = "Steer the running agent, or queue a follow-up";
pub(crate) const SEND: &str = "Send";
pub(crate) const STEER: &str = "Steer the run";
pub(crate) const QUEUE: &str = "Queue for after the run";
pub(crate) const STOP: &str = "Stop";
pub(crate) const ATTACH: &str = "Attach files";
pub(crate) const ATTACH_PROMPT: &str = "Attach";
pub(crate) const REMOVE_ATTACHMENT: &str = "Remove attachment";
pub(crate) const HINT_IDLE: &str = "Enter to send · Shift+Enter for a new line";
pub(crate) const HINT_IDLE_CTRL: &str = "Ctrl+Enter to send · Enter for a new line";
pub(crate) const HINT_STEER: &str = "Enter steers the run · Ctrl+Enter queues";
pub(crate) const HINT_QUEUE: &str = "Enter queues · Ctrl+Enter steers the run";
pub(crate) const NO_MODEL: &str =
    "Your instance offers no model right now, so messages cannot be sent.";
pub(crate) const NO_COMMANDS: &str = "No matching command.";
pub(crate) const NO_FILES: &str = "No matching file.";
pub(crate) const BUILTIN: &str = "Built in";
pub(crate) const PASTE_FAILED: &str = "The pasted image could not be attached.";
pub(crate) const DIALOG_FAILED: &str = "The file picker could not be opened.";
pub(crate) const PASTED_IMAGE: &str = "Pasted image";
