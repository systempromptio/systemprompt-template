//! The English text of the Drive.
//!
//! The screen's browsing, documents, preview and tabs, comments, sharing and
//! the copy for every Drive failure are here; the sync panel's text is in
//! [`sync`], the settings pane's in [`settings`].

pub mod settings;
pub mod sync;

use chrono::{DateTime, Datelike as _, Local};
use sp_drive::DocumentKind;
use sp_drive::view::DriveFailure;

use crate::model::BrowseScope;

pub const DRIVE_NAME: &str = "Drive";
pub const DRIVE_TITLE: &str = "Drive";

pub const WITHHELD_TITLE: &str = "The Drive is off";
pub const WITHHELD_BODY: &str =
    "No instance is connected, or the connected instance does not enable the Drive for this app.";
pub const NO_DRIVE_TITLE: &str = "This instance has no Drive";
pub const NO_DRIVE_BODY: &str = "Ask your administrator to enable documents on the instance.";

pub const SCOPE_MINE: &str = "Mine";
pub const SCOPE_SHARED: &str = "Shared";
pub const SCOPE_RECENT: &str = "Recent";
pub const ALL_DOCUMENTS: &str = "All documents";
pub const FOLDERS: &str = "Folders";
pub const NO_FOLDERS: &str = "No folders yet.";
pub const EXPAND_FOLDER: &str = "Show subfolders";
pub const COLLAPSE_FOLDER: &str = "Hide subfolders";

pub const SEARCH: &str = "Search documents";
pub const SORT_UPDATED: &str = "Last updated";
pub const SORT_NAME: &str = "Name";
pub const SORT_SIZE: &str = "Size";
pub const REFRESH: &str = "Refresh";
pub const NEW: &str = "New";
pub const NEW_FOLDER: &str = "Folder";
pub const NEW_DOCUMENT_HEADING: &str = "New document";
pub const IMPORT: &str = "Import a file";
pub const IMPORT_PROMPT: &str = "Import into the Drive";
pub const LOAD_MORE: &str = "Load more";
pub const NO_DOCUMENTS_TITLE: &str = "No documents here";
pub const NO_DOCUMENTS_BODY: &str = "Create a document or import a file to start.";
pub const NO_MATCHES: &str = "No documents match your search.";

pub const NAME_LABEL: &str = "Name";
pub const NEW_FOLDER_TITLE: &str = "New folder";
pub const NEW_FOLDER_PLACEHOLDER: &str = "Folder name";
pub const NEW_DOCUMENT_PLACEHOLDER: &str = "Document name";
pub const CREATE: &str = "Create";
pub const CANCEL: &str = "Cancel";
pub const FOLDER_CREATED: &str = "Folder created.";
pub const DOCUMENT_CREATED: &str = "Document created.";

pub const UPLOADING: &str = "Uploading";
pub const CANCEL_UPLOAD: &str = "Cancel upload";
pub const UPLOADED: &str = "File imported.";
pub const UPLOAD_CANCELLED: &str = "The upload was cancelled.";
pub const FILE_TOO_LARGE: &str = "That file is too large to import.";
pub const FILE_UNREADABLE: &str = "That file could not be read.";

pub const NOTHING_SELECTED_TITLE: &str = "Select a document";
pub const NOTHING_SELECTED_BODY: &str = "Its text, comments, revisions and sharing appear here.";
pub const TAB_PREVIEW: &str = "Preview";
pub const TAB_COMMENTS: &str = "Comments";
pub const TAB_REVISIONS: &str = "Revisions";
pub const TAB_SHARING: &str = "Sharing";
pub const OPEN_IN_BROWSER: &str = "Open in browser";
pub const ATTACH: &str = "Attach to the composer";
pub const COPY_CODE: &str = "Copy";
pub const PREVIEW_EMPTY: &str = "This document has no text to show.";
pub const PREVIEW_TRUNCATED: &str =
    "The preview stops here. Open the document in the browser to read the rest.";
pub const PREVIEW_UNAVAILABLE: &str = "This document cannot be previewed here.";

pub const NO_COMMENTS: &str = "No comments yet.";
pub const COMMENT_PLACEHOLDER: &str = "Add a comment";
pub const COMMENT: &str = "Comment";
pub const REPLY: &str = "Reply";
pub const RESOLVE: &str = "Resolve";
pub const RESOLVED: &str = "Resolved";
pub const CANCEL_REPLY: &str = "Cancel reply";
pub const COMMENT_ON: &str = "On";
pub const COMMENT_NEEDS_TEXT: &str = "Write something first.";
pub const COMMENT_NEEDS_ANCHOR: &str = "This document has no paragraph to comment on.";

pub const NO_REVISIONS: &str = "No revisions to show.";
pub const REVISION_BY_AGENT: &str = "Agent";
pub const REVISION_BY_PERSON: &str = "Person";
pub const HEAD_REVISION: &str = "Current";

pub const OWNER: &str = "Owner";
pub const YOUR_ACCESS: &str = "Your access";
pub const PEOPLE_WITH_ACCESS: &str = "People with access";
pub const INHERITED_ACCESS: &str = "From folders";
pub const AGENT_ACCESS: &str = "Agent access";
pub const NOT_SHARED: &str = "Not shared with anyone else.";
pub const MANAGE_SHARING: &str = "Manage sharing in the browser";

pub const fn failure(kind: DriveFailure) -> &'static str {
    match kind {
        DriveFailure::SignedOut => "Sign in to your instance to use the Drive.",
        DriveFailure::Unreachable => "The instance could not be reached. Try again in a moment.",
        DriveFailure::NotFound => "That document or folder is no longer there.",
        DriveFailure::Forbidden => "You do not have access to that.",
        DriveFailure::PersonOnly => {
            "Only you can do this, in the instance's Drive in your browser."
        },
        DriveFailure::SuggestOnly => "Changes to this document can only be suggested.",
        DriveFailure::Conflict => "The document changed meanwhile. Refresh and try again.",
        DriveFailure::Cancelled => "The transfer was cancelled.",
        DriveFailure::Invalid => "That request is not valid.",
        DriveFailure::Failed => "The Drive request failed.",
    }
}

pub const fn scope(scope: BrowseScope) -> &'static str {
    match scope {
        BrowseScope::Mine => SCOPE_MINE,
        BrowseScope::Shared => SCOPE_SHARED,
        BrowseScope::Recent => SCOPE_RECENT,
    }
}

pub const fn kind(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Markdown => "Markdown document",
        DocumentKind::Text => "Plain text",
        DocumentKind::Html => "Web page",
        DocumentKind::Word => "Word document",
        DocumentKind::Spreadsheet => "Spreadsheet",
        DocumentKind::Presentation => "Presentation",
    }
}

pub fn uploading(name: &str) -> String {
    format!("{UPLOADING} {name}")
}

pub fn attach_reference(name: &str, id: &str) -> String {
    format!("Use the Drive document \"{name}\" (document id {id}). ")
}

pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

pub fn when(raw: &str, now: DateTime<Local>) -> String {
    let Ok(parsed) = DateTime::parse_from_rfc3339(raw) else {
        return raw.to_owned();
    };
    let local = parsed.with_timezone(&Local);
    if local.date_naive() == now.date_naive() {
        local.format("%H:%M").to_string()
    } else if local.year() == now.year() {
        local.format("%d %b").to_string()
    } else {
        local.format("%d %b %Y").to_string()
    }
}
