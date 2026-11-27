//! The English text of the docks: the permission request and the questions a
//! session waits on.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

pub(crate) const PERMISSION_TITLE: &str = "Permission needed";
pub(crate) const ALLOW_ONCE: &str = "Allow once";
pub(crate) const ALLOW_ALWAYS: &str = "Always allow";
pub(crate) const REJECT: &str = "Reject";
pub(crate) const REJECT_WITH_FEEDBACK: &str = "Reject with feedback";
pub(crate) const SEND_REJECTION: &str = "Send";
pub(crate) const CANCEL: &str = "Cancel";
pub(crate) const FEEDBACK_PLACEHOLDER: &str = "Tell the agent what to do instead";
pub(crate) const PATTERNS: &str = "Applies to";
pub(crate) const KEY_ENTER: &str = "Enter";
pub(crate) const KEY_ALWAYS: &str = "A";
pub(crate) const KEY_REJECT: &str = "Esc";
pub(crate) const KEY_FEEDBACK: &str = "R";

pub(crate) const QUESTION_TITLE: &str = "The agent has a question";
pub(crate) const QUESTIONS_TITLE: &str = "The agent has questions";
pub(crate) const OWN_ANSWER: &str = "Or type your own answer";
pub(crate) const SUBMIT: &str = "Submit";
pub(crate) const DISMISS: &str = "Dismiss";
pub(crate) const PICK_SEVERAL: &str = "Choose any";

pub(crate) fn waiting(more: usize) -> String {
    if more == 1 {
        "1 more waiting".to_owned()
    } else {
        format!("{more} more waiting")
    }
}

pub(crate) fn always_covers(permission: &str, always: &[String]) -> String {
    if always.is_empty() || always.iter().any(|pattern| pattern == "*") {
        return format!(
            "Always allow approves every {permission} request for the rest of this session."
        );
    }
    format!(
        "Always allow approves {permission} for {} for the rest of this session.",
        always.join(", ")
    )
}
