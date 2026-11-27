use std::path::{Path, PathBuf};

use systemprompt_drive::default_drive_root;
use systemprompt_harness::extension::SettingsError;
use systemprompt_harness::members::drive::{DriveExtension, DriveSettings};
use systemprompt_harness_views::drive::model::{SyncFolder, default_folder};
use systemprompt_harness_views::drive::strings::settings as text;

#[test]
fn a_chosen_folder_wins_over_the_default() {
    let chosen = PathBuf::from("/data/My Drive");
    let folder = SyncFolder::resolve(
        Ok(DriveSettings {
            folder: Some(chosen.clone()),
        }),
        Some(Path::new("/home/u/Documents/Drive")),
    );
    assert_eq!(folder, SyncFolder::Chosen(chosen.clone()));
    assert_eq!(folder.path(), Some(chosen.as_path()));
    assert!(folder.can_reset());
}

#[test]
fn without_a_choice_the_default_is_shown_and_there_is_nothing_to_reset() {
    let default = Path::new("/home/u/Documents/Drive");
    let folder = SyncFolder::resolve(Ok(DriveSettings::default()), Some(default));
    assert_eq!(folder, SyncFolder::Default(default.to_path_buf()));
    assert!(!folder.can_reset());
    let unknown = SyncFolder::resolve(Ok(DriveSettings::default()), None);
    assert_eq!(unknown, SyncFolder::Unknown);
    assert_eq!(unknown.path(), None);
}

#[test]
fn an_unreadable_choice_does_not_fall_back_to_the_default() {
    let error = SettingsError::NotAnObject {
        extension: DriveExtension::ID,
    };
    let folder = SyncFolder::resolve(Err(&error), Some(Path::new("/home/u/Documents/Drive")));
    assert_eq!(folder, SyncFolder::Unreadable);
    assert_eq!(folder.path(), None);
    assert!(folder.can_reset());
    assert_ne!(
        text::folder_state(&folder),
        text::folder_state(&SyncFolder::Unknown)
    );
}

#[test]
fn the_default_folder_is_the_products_folder_under_documents() {
    let documents = Path::new("/home/u/Documents");
    assert_eq!(
        default_folder(Some(documents), "Harness"),
        Some(default_drive_root(documents, "Harness"))
    );
    assert_eq!(default_folder(None, "Harness"), None);
}
