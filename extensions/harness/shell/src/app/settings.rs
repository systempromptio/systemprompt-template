//! The shell's own persisted preferences.
//!
//! They cover appearance, recent projects, the last session per project, window
//! geometry, the verdict switch, composer preferences and each extension's own
//! preferences object, kept under its extension id. They are owned and saved by
//! [`SettingsStore`](super::settings_store::SettingsStore).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use harness_ids::ExtensionId;
use harness_store::SessionId;
use harness_ui::Scheme;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

const VERSION: u32 = 1;
const RECENT_LIMIT: usize = 10;
const MIN_WINDOW_EDGE: f32 = 320.0;
const MAX_WINDOW_EDGE: f32 = 20_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryPref {
    #[default]
    Steer,
    Queue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ComposerPrefs {
    pub enter_sends: bool,
    pub delivery: DeliveryPref,
}

impl Default for ComposerPrefs {
    fn default() -> Self {
        Self {
            enter_sends: true,
            delivery: DeliveryPref::Steer,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectPrefs {
    pub last_session: Option<SessionId>,
    pub(crate) model: Option<String>,
    pub(crate) agent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub maximized: bool,
}

impl WindowGeometry {
    pub fn is_sane(&self) -> bool {
        let finite = [self.x, self.y, self.width, self.height]
            .iter()
            .all(|value| value.is_finite());
        let edge = |value: f32| (MIN_WINDOW_EDGE..=MAX_WINDOW_EDGE).contains(&value);
        finite && edge(self.width) && edge(self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub version: u32,
    pub(crate) scheme: String,
    pub last_project: Option<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
    pub(crate) projects: BTreeMap<String, ProjectPrefs>,
    pub window: Option<WindowGeometry>,
    pub sidebar_open: bool,
    pub verdict_enabled: bool,
    pub composer: ComposerPrefs,
    // JSON: each extension's own preferences object, stored as the extension wrote it.
    pub extensions: BTreeMap<ExtensionId, Map<String, Value>>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            version: VERSION,
            scheme: Scheme::System.key().to_owned(),
            last_project: None,
            recent_projects: Vec::new(),
            projects: BTreeMap::new(),
            window: None,
            sidebar_open: true,
            verdict_enabled: true,
            composer: ComposerPrefs::default(),
            extensions: BTreeMap::new(),
        }
    }
}

impl UiSettings {
    pub fn scheme(&self) -> Scheme {
        Scheme::from_key(&self.scheme).unwrap_or_default()
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        scheme.key().clone_into(&mut self.scheme);
    }

    pub(crate) fn project(&self, directory: &Path) -> ProjectPrefs {
        self.projects
            .get(&project_key(directory))
            .cloned()
            .unwrap_or_default()
    }

    pub fn project_mut(&mut self, directory: &Path) -> &mut ProjectPrefs {
        self.projects.entry(project_key(directory)).or_default()
    }

    pub fn remember_project(&mut self, directory: &Path) {
        let key = project_key(directory);
        self.recent_projects
            .retain(|recent| project_key(recent) != key);
        self.recent_projects.insert(0, directory.to_path_buf());
        self.recent_projects.truncate(RECENT_LIMIT);
        self.last_project = Some(directory.to_path_buf());
    }

    pub fn forget_project(&mut self, directory: &Path) {
        let key = project_key(directory);
        self.recent_projects
            .retain(|recent| project_key(recent) != key);
        if self
            .last_project
            .as_deref()
            .is_some_and(|last| project_key(last) == key)
        {
            self.last_project = None;
        }
    }
}

pub(crate) fn project_key(directory: &Path) -> String {
    let text = directory.to_string_lossy();
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text.into_owned()
    }
}
