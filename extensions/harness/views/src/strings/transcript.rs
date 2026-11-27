//! The English text of the transcript: the session header, message actions,
//! tool rows, the changes panel and the status lines under the conversation.

use systemprompt_engine::run::ErrorKind;
use systemprompt_engine::store::TodoStatus;

pub(crate) const UNTITLED: &str = "Untitled session";
pub(crate) const EMPTY_TITLE: &str = "Nothing here yet";
pub(crate) const EMPTY_BODY: &str = "Write a message below to start.";

pub(crate) const YOU: &str = "You";
pub(crate) const COPY: &str = "Copy";
pub(crate) const COPIED: &str = "Copied to the clipboard.";
pub(crate) const COPY_CODE: &str = "Copy code";
pub(crate) const REVERT: &str = "Undo from here";
pub(crate) const FORK: &str = "Fork from here";

pub(crate) const THINKING: &str = "Thinking";
pub(crate) const THINKING_LIVE: &str = "Thinking…";
pub(crate) const WORKING: &str = "Working…";
pub(crate) const SUMMARY: &str = "Summary of the earlier conversation";
pub(crate) const COMPACTED: &str = "The conversation was compacted to free up context";
pub(crate) const COMPACTED_AUTO: &str =
    "The conversation was compacted automatically to free up context";
pub(crate) const STOPPED: &str = "Stopped.";
pub(crate) const ATTACHMENT: &str = "Attachment";

pub(crate) const INPUT: &str = "Input";
pub(crate) const OUTPUT: &str = "Output";
pub(crate) const NO_OUTPUT: &str = "No output.";
pub(crate) const EXIT_CODE: &str = "Exit code";
pub(crate) const TIMED_OUT: &str = "Timed out";
pub(crate) const POLICY_DENIED: &str = "Blocked by policy";
pub(crate) const RULE_DENIED: &str = "Denied by a rule";
pub(crate) const REJECTED: &str = "Rejected by you";
pub(crate) const FAILED: &str = "Failed";
pub(crate) const NO_REASON: &str = "No reason was given.";
pub(crate) const OPEN_SUBTASK: &str = "Open subtask";
pub(crate) const SUBTASK_STARTING: &str = "Starting…";

pub(crate) const CHANGES_TITLE: &str = "Changed files";
pub(crate) const CHANGES_EMPTY: &str = "No files changed in this session.";
pub(crate) const CHANGES_REFRESH: &str = "Refresh";
pub(crate) const CHANGES_CLOSE: &str = "Close changes";
pub(crate) const BINARY_FILE: &str = "Binary file";

pub(crate) const DISMISS: &str = "Dismiss";
pub(crate) const REVERTED: &str = "Later messages were undone and their file changes rolled back.";
pub(crate) const UNREVERT: &str = "Restore them";
pub(crate) const JUMP_TO_LATEST: &str = "Jump to latest";

pub(crate) fn duration(ms: i64) -> String {
    let ms = ms.max(0);
    if ms < 1_000 {
        return format!("{ms} ms");
    }
    let seconds = ms / 1_000;
    if seconds < 60 {
        return format!("{}.{} s", seconds, (ms % 1_000) / 100);
    }
    format!("{} m {:02} s", seconds / 60, seconds % 60)
}

pub(crate) fn folded(lines: usize) -> String {
    if lines == 1 {
        "1 unchanged line".to_owned()
    } else {
        format!("{lines} unchanged lines")
    }
}

pub(crate) fn files_changed(files: u64) -> String {
    if files == 1 {
        "1 file".to_owned()
    } else {
        format!("{files} files")
    }
}

pub(crate) fn todo_progress(done: usize, total: usize) -> String {
    format!("{done}/{total} done")
}

pub(crate) const fn todo_marker(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::Pending => "Pending",
        TodoStatus::InProgress => "In progress",
        TodoStatus::Completed => "Done",
        TodoStatus::Cancelled => "Cancelled",
    }
}

pub(crate) fn subtask_progress(steps: usize, current: Option<&str>) -> String {
    let count = if steps == 1 {
        "1 step".to_owned()
    } else {
        format!("{steps} steps")
    };
    match current {
        Some(current) => format!("{count} · {current}"),
        None => count,
    }
}

pub(crate) fn retry(attempt: u32, seconds: u64, reason: &str) -> String {
    format!("Retrying (attempt {attempt}) in {seconds} s — {reason}")
}

pub(crate) const fn failure_title(kind: ErrorKind) -> &'static str {
    crate::strings::foundation::failure(kind)
}
