//! The English text of the shell's frame: titlebar, sidebar and its session
//! menu, home screen, notices and the empty Settings page.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use harness_engine::ErrorKind;

use crate::state::DayBucket;

pub(crate) const NEW_SESSION: &str = "New session";
pub(crate) const UNTITLED_SESSION: &str = "Untitled session";
pub(crate) const SEARCH_SESSIONS: &str = "Search sessions";
pub(crate) const NO_SESSIONS_TITLE: &str = "No sessions yet";
pub(crate) const NO_SESSIONS_BODY: &str = "Start one with the composer below.";
pub(crate) const NO_MATCHES: &str = "No sessions match your search.";
pub(crate) const SESSIONS: &str = "Sessions";
pub(crate) const SHOW_SUBTASKS: &str = "Show subtasks";
pub(crate) const HIDE_SUBTASKS: &str = "Hide subtasks";

pub(crate) const TODAY: &str = "Today";
pub(crate) const YESTERDAY: &str = "Yesterday";
pub(crate) const PREVIOUS_WEEK: &str = "Previous 7 days";
pub(crate) const PREVIOUS_MONTH: &str = "Previous 30 days";
pub(crate) const OLDER: &str = "Older";

pub(crate) const OPEN_FOLDER: &str = "Open folder…";
pub(crate) const OPEN_FOLDER_PROMPT: &str = "Open project";
pub(crate) const RECENT_PROJECTS: &str = "Recent projects";
pub(crate) const NO_PROJECT: &str = "Open a folder to start a session.";
pub(crate) const NO_PROJECT_NAME: &str = "No project";

pub(crate) const SETTINGS: &str = "Settings";
pub(crate) const TOGGLE_SIDEBAR: &str = "Toggle sidebar";
pub(crate) const SIGNED_OUT: &str = "Not signed in";

pub(crate) const SESSION_ACTIONS: &str = "Session actions";
pub(crate) const COMPACT_SESSION: &str = "Compact now";
pub(crate) const DELETE_SESSION: &str = "Delete…";
pub(crate) const DELETE_SESSION_TITLE: &str = "Delete this session?";
pub(crate) const DELETE: &str = "Delete";
pub(crate) const CANCEL: &str = "Cancel";
pub(crate) const SESSION_COMPACTED: &str = "The session was compacted.";

pub(crate) const HOME_TITLE: &str = "What should we work on?";
pub(crate) const HOME_HINT: &str = "Describe a change, ask about the code, or type / for commands.";

pub(crate) const PROJECT_LOAD_FAILED: &str = "The project could not be opened.";
pub(crate) const LINK_NOT_UNDERSTOOD: &str = "That link could not be opened.";
pub(crate) const COMMAND_PALETTE_UNAVAILABLE: &str = "The command palette is not available yet.";

pub(crate) const STARTUP_FAILED_TITLE: &str = "The app could not start";
pub(crate) const STARTUP_FAILED_BODY: &str =
    "Details were written to the log in the app's data folder.";
pub(crate) const QUIT: &str = "Quit";

pub(crate) const PLACEHOLDER_SETTINGS: &str = "Choose a settings page.";

pub(crate) const fn day(bucket: DayBucket) -> &'static str {
    match bucket {
        DayBucket::Today => TODAY,
        DayBucket::Yesterday => YESTERDAY,
        DayBucket::PreviousWeek => PREVIOUS_WEEK,
        DayBucket::PreviousMonth => PREVIOUS_MONTH,
        DayBucket::Older => OLDER,
    }
}

pub(crate) const fn failure(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotSignedIn => "Sign in to an instance to use a model.",
        ErrorKind::ModelNotOffered => "That model is not offered by your instance.",
        ErrorKind::InvalidInput => "That request is not valid.",
        ErrorKind::NotFound => "That session no longer exists.",
        ErrorKind::Busy => "The session is busy.",
        ErrorKind::Governance => "Your instance refused the request.",
        ErrorKind::Model => "The model failed to answer.",
        ErrorKind::ContextOverflow => "The conversation is too long for the model.",
        ErrorKind::Aborted => "The run was stopped.",
        ErrorKind::Config => "The project configuration could not be read.",
        ErrorKind::Storage => "The session store failed.",
        ErrorKind::Workspace => "The workspace snapshot failed.",
        ErrorKind::Unavailable => "The service could not be reached.",
        ErrorKind::Internal => "Something went wrong.",
    }
}

pub(crate) fn delete_session_body(title: &str) -> String {
    format!(
        "“{title}” and its subtasks are removed for good. Files the session changed stay as they are."
    )
}

pub(crate) fn home_project(name: &str) -> String {
    format!("in {name}")
}
