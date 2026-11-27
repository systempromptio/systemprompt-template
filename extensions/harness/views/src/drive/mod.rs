//! The Drive's screens, as a shell extension.
//!
//! [`DriveShellExtension`] (id `drive`, the id of the instance-side
//! [`DriveExtension`](systemprompt_harness::members::drive::DriveExtension))
//! adds the Drive screen, entered from the titlebar: the documents and folders
//! of the instance the Drive handle follows, a document's preview, comments,
//! revisions and sharing, creating folders and documents, importing a file,
//! attaching a document to the composer, and the local folder the sync keeps.
//! It also adds the Drive settings pane, where the user chooses the sync's base
//! folder with the native folder picker or goes back to the default under
//! Documents.
//!
//! Every view reads the handle and its sync's status feed when it draws and
//! shows a withheld Drive, a stopped sync or a failed call explicitly. The
//! shell shows these contributions only while the instance admits the
//! extension.

mod bridge;
mod extension;
pub mod items;
pub mod model;
mod settings_pane;
pub mod strings;
mod view;

pub use extension::{DRIVE_SCREEN, DRIVE_SETTINGS_PANE, DriveShellExtension};
