//! The English text of the local folder panel.
//!
//! Where the sync stands, its counts and problems, held deletes, conflicts and
//! recent activity.

use chrono::{DateTime, Local, Utc};
use systemprompt_drive::sync::{HeldReason, SyncProblemKind};
use systemprompt_drive::{ActivityKind, SyncPhase};

use super::when;

pub const TITLE: &str = "Local folder";
pub const OPEN_FOLDER: &str = "Open the folder";
pub const PAUSE: &str = "Pause";
pub const RESUME: &str = "Resume";
pub const SYNC_NOW: &str = "Sync now";
pub const DETAILS: &str = "Show details";
pub const HIDE_DETAILS: &str = "Hide details";
pub const CONFLICTS: &str = "Conflicts";
pub const SHOW_COPY: &str = "Show the kept copy";
pub const OPEN_FILE: &str = "Open the file";
pub const IN_REVIEW: &str = "Sent for review";
pub const PUT_BACK: &str = "Put back or not placed";
pub const RECENT: &str = "Recent activity";
pub const NO_ACTIVITY: &str = "Nothing has synced yet.";
pub const APPLY_DELETES: &str = "Delete on the instance";
pub const RESTORE_DELETES: &str = "Restore the files";
pub const CHOICE_FAILED: &str = "Your choice could not be saved. Try again.";

pub const fn phase(phase: SyncPhase) -> &'static str {
    match phase {
        SyncPhase::Off => "Not syncing",
        SyncPhase::Idle => "Up to date",
        SyncPhase::Syncing => "Syncing",
        SyncPhase::Paused => "Paused",
        SyncPhase::Disabled => "The instance has no Drive to sync",
        SyncPhase::Error => "Sync stopped",
    }
}

pub const fn problem(kind: SyncProblemKind) -> &'static str {
    match kind {
        SyncProblemKind::SignedOut => "The instance refused this sign-in. Sign in again.",
        SyncProblemKind::NoDeviceToken => "This sign-in cannot sync files. Sign out and in again.",
        SyncProblemKind::Unreachable => {
            "The instance could not be reached; the sync retries on its own."
        },
        SyncProblemKind::Refused => "The instance refused a file.",
        SyncProblemKind::LocalFiles => "A file in the folder could not be read or written.",
        SyncProblemKind::Failed => "The last sync failed.",
    }
}

pub fn held(reason: HeldReason, count: usize) -> String {
    match reason {
        HeldReason::RootEmpty => {
            format!("The folder looks emptied: {count} deleted file(s) are held until you choose.")
        },
        HeldReason::Bulk => {
            format!("{count} files were deleted at once; they are held until you choose.")
        },
    }
}

pub fn activity(kind: &ActivityKind) -> String {
    match kind {
        ActivityKind::Downloaded => "Downloaded".to_owned(),
        ActivityKind::Uploaded => "Uploaded".to_owned(),
        ActivityKind::Merged => "Merged with a newer version".to_owned(),
        ActivityKind::SentForReview => "Sent for review".to_owned(),
        ActivityKind::Conflict { kept_as } => format!("Conflict, your copy kept as {kept_as}"),
        ActivityKind::Moved { from } => format!("Moved from {from}"),
        ActivityKind::Trashed => "Moved to the instance's trash".to_owned(),
        ActivityKind::DeletedRemotely => "Deleted on the instance".to_owned(),
        ActivityKind::Restored => "Put back".to_owned(),
        ActivityKind::Refused { .. } => "Refused by the instance".to_owned(),
        ActivityKind::NotPlaced { .. } => "Could not be placed in the folder".to_owned(),
    }
}

pub fn counts(synced: usize, pending: usize) -> String {
    if pending == 0 {
        format!("{synced} file(s) synced")
    } else {
        format!("{synced} file(s) synced, {pending} waiting to upload")
    }
}

pub fn synced_at(at: DateTime<Utc>, now: DateTime<Local>) -> String {
    format!("Last synced {}", when(&at.to_rfc3339(), now))
}
