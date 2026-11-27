//! Every directory the app writes to, all under the build's data root, and the
//! choice of the project directory to open at start.
//!
//! Nothing here reads XDG variables or another product's directories: the root
//! is `sp_brand::identity().data_root()` (`%LOCALAPPDATA%\<vendor>\<product>[
//! channel]` on Windows).
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::io;
use std::path::{Path, PathBuf};

const STORE_FILE: &str = "sessions.db";
const ACCOUNTS_FILE: &str = "accounts.json";
const UI_SETTINGS_FILE: &str = "ui-settings.json";
#[cfg(unix)]
const INSTANCE_SOCKET: &str = "instance.sock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub(crate) root: PathBuf,
    pub data: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    pub state: PathBuf,
    pub tmp: PathBuf,
    pub logs: PathBuf,
    pub(crate) home: PathBuf,
    pub documents: Option<PathBuf>,
}

impl AppPaths {
    pub fn under(root: PathBuf, home: PathBuf) -> Self {
        Self {
            data: root.join("data"),
            config: root.join("config"),
            cache: root.join("cache"),
            state: root.join("state"),
            tmp: root.join("tmp"),
            logs: root.join("logs"),
            root,
            home,
            documents: None,
        }
    }

    pub(crate) fn ensure(&self) -> io::Result<()> {
        for dir in [
            &self.data,
            &self.config,
            &self.cache,
            &self.state,
            &self.tmp,
            &self.logs,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn engine_paths(&self) -> harness_config::Paths {
        harness_config::Paths {
            home: self.home.clone(),
            data: self.data.clone(),
            config: self.config.clone(),
            cache: self.cache.clone(),
            state: self.state.clone(),
            tmp: self.tmp.clone(),
        }
    }

    pub fn store_file(&self) -> PathBuf {
        self.data.join(STORE_FILE)
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.state.join(ACCOUNTS_FILE)
    }

    pub fn ui_settings_file(&self) -> PathBuf {
        self.state.join(UI_SETTINGS_FILE)
    }

    #[cfg(unix)]
    pub(crate) fn instance_socket(&self) -> PathBuf {
        self.state.join(INSTANCE_SOCKET)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Forbidden {
    pub system_root: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
}

impl Forbidden {
    pub fn rejects(&self, candidate: &Path) -> bool {
        let candidate = folded(candidate);
        let inside_system = self
            .system_root
            .as_deref()
            .is_some_and(|root| is_within(&candidate, &folded(root)));
        let is_install = self
            .install_dir
            .as_deref()
            .is_some_and(|dir| is_within(&candidate, &folded(dir)));
        inside_system || is_install
    }
}

pub fn choose_directory(
    requested: Option<PathBuf>,
    last_project: Option<PathBuf>,
    home: &Path,
    forbidden: &Forbidden,
    is_dir: impl Fn(&Path) -> bool,
) -> PathBuf {
    [requested, last_project]
        .into_iter()
        .flatten()
        .find(|candidate| {
            candidate.is_absolute() && is_dir(candidate) && !forbidden.rejects(candidate)
        })
        .unwrap_or_else(|| home.to_path_buf())
}

pub(crate) fn absolute(path: &Path) -> Option<PathBuf> {
    std::path::absolute(path).ok()
}

pub fn display_name(directory: &Path) -> String {
    directory.file_name().map_or_else(
        || directory.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn folded(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/").to_lowercase();
    text.trim_end_matches('/').to_owned()
}

fn is_within(candidate: &str, root: &str) -> bool {
    candidate == root
        || candidate
            .strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('/'))
}
