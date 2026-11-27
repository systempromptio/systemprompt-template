//! The English text of the Drive settings pane.
//!
//! The base folder of the local sync, where it comes from, and choosing or
//! resetting it.

use crate::drive::model::SyncFolder;

pub const TITLE: &str = "Drive";
pub const DESCRIPTION: &str =
    "The folder on this computer that the Drive keeps in sync with your instance.";
pub const FOLDER_TITLE: &str = "Drive folder";
pub const FOLDER_NOTE: &str = "Each instance you sign in to gets its own folder inside this one.";
pub const CHOOSE: &str = "Choose folder";
pub const CHOOSE_PROMPT: &str = "Choose the Drive folder";
pub const USE_DEFAULT: &str = "Use the default folder";
pub const CHOSEN: &str = "Chosen";
pub const DEFAULT: &str = "Default";
pub const IN_USE: &str = "Syncing into";
pub const SAVE_FAILED: &str = "The folder could not be saved. Try again.";
pub const PICKER_FAILED: &str = "The folder picker could not be opened.";
pub const DISMISS: &str = "Dismiss";
pub const WITHHELD: &str =
    "The Drive is off for this instance. The folder is used once an instance enables the Drive.";

pub const fn folder_state(folder: &SyncFolder) -> &'static str {
    match folder {
        SyncFolder::Chosen(_) => "You chose this folder.",
        SyncFolder::Default(_) => "No folder is chosen, so the Drive uses this default.",
        SyncFolder::Unknown => {
            "No folder is chosen and this computer has no Documents folder, so the Drive does not sync. Choose a folder to start."
        },
        SyncFolder::Unreadable => {
            "The saved folder could not be read, so the Drive does not sync. Choose a folder again or use the default."
        },
    }
}
