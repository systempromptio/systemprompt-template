//! Which folder the local sync uses as its base, as the settings pane shows it:
//! the folder the user chose, else the product's default under Documents. A
//! preference that does not decode keeps the sync off rather than falling back.

use std::path::{Path, PathBuf};

use sp_drive::{DriveSettings, default_drive_root};
use sp_extension::SettingsError;

/// The base folder of the local sync.
///
/// `Unknown` is no choice and no Documents directory to put the default in;
/// `Unreadable` is a stored choice that does not decode. In both the sync is
/// off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncFolder {
    Chosen(PathBuf),
    Default(PathBuf),
    Unknown,
    Unreadable,
}

impl SyncFolder {
    pub fn resolve(
        preference: Result<DriveSettings, &SettingsError>,
        default: Option<&Path>,
    ) -> Self {
        match preference {
            Ok(DriveSettings {
                folder: Some(folder),
            }) => Self::Chosen(folder),
            Ok(DriveSettings { folder: None }) => default.map_or(Self::Unknown, |default| {
                Self::Default(default.to_path_buf())
            }),
            Err(_) => Self::Unreadable,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Chosen(path) | Self::Default(path) => Some(path),
            Self::Unknown | Self::Unreadable => None,
        }
    }

    pub const fn can_reset(&self) -> bool {
        matches!(self, Self::Chosen(_) | Self::Unreadable)
    }
}

pub fn default_folder(documents: Option<&Path>, product: &str) -> Option<PathBuf> {
    documents.map(|documents| default_drive_root(documents, product))
}
