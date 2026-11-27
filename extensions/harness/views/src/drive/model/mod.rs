//! The Drive's pure presentation model.
//!
//! What is being browsed, the folder tree as rows, the preview text built from
//! a document's outline, links into the instance and the local folder, what
//! the screen is pointed at and which folder the sync uses. Nothing here
//! touches the toolkit or the network.

mod browse;
mod folders;
mod links;
mod preview_text;
mod settings;
mod target;

pub use browse::{Browse, BrowseScope, DriveLoad, SortKey, sort_documents};
pub use folders::{FolderRow, folder_rows};
pub use links::{local_path, web_link};
pub use preview_text::{AnchorChoice, EXCERPT_CHARS, PREVIEW_LIMIT, PreviewText};
pub use settings::{SyncFolder, default_folder};
pub use target::DriveTarget;
