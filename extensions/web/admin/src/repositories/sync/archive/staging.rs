//! Uploaded archives held between upload and apply.
//!
//! A stage is a scratch object with a thirty-minute life: the plane files
//! as text, the manifest, and every other entry described. It lives in this
//! process's memory rather than a table because nothing about it needs to
//! outlive the operator's tab — and a table would need a schema, a sweeper
//! and a prepare run for a preview. The one consequence: on a deployment
//! with more than one instance, the preview URL must reach the instance
//! that staged it. This installation runs one; revisit with a
//! `sync_import_stages` table if that changes.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use super::manifest::ArchiveManifest;

pub const STAGE_TTL_MINUTES: i64 = 30;
const MAX_STAGES: usize = 8;

/// An archive entry the instance does not project: named for what it is.
#[derive(Debug, Clone, Serialize)]
pub struct OtherEntry {
    pub path: String,
    pub bytes: usize,
    pub hash: String,
    pub kind_id: Option<&'static str>,
    pub kind_label: Option<&'static str>,
}

/// One staged upload.
#[derive(Debug, Clone)]
pub struct StagedArchive {
    pub id: String,
    pub actor: String,
    pub created_at: DateTime<Utc>,
    pub manifest: Option<ArchiveManifest>,
    pub manifest_error: Option<String>,
    pub planes: BTreeMap<&'static str, String>,
    pub other: Vec<OtherEntry>,
}

impl StagedArchive {
    #[must_use]
    pub fn new(
        actor: &str,
        manifest: Option<ArchiveManifest>,
        manifest_error: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().simple().to_string(),
            actor: actor.to_owned(),
            created_at: Utc::now(),
            manifest,
            manifest_error,
            planes: BTreeMap::new(),
            other: Vec::new(),
        }
    }

    #[must_use]
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.created_at + Duration::minutes(STAGE_TTL_MINUTES)
    }

    fn expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at()
    }
}

/// The process-wide store of staged uploads.
#[derive(Debug, Clone, Default)]
pub struct StagingStore(Arc<Mutex<HashMap<String, StagedArchive>>>);

impl StagingStore {
    // Why: the JSON API and the SSR pages are separate routers built in
    // separate places; one process-wide store is what lets a stage created
    // through one be read by the other.
    pub fn global() -> &'static Self {
        static STORE: OnceLock<StagingStore> = OnceLock::new();
        STORE.get_or_init(Self::default)
    }

    // Why: expired stages are swept on every put, and the oldest is dropped
    // past the cap, so the store is bounded without a background task.
    pub fn put(&self, stage: StagedArchive) -> String {
        let id = stage.id.clone();
        if let Ok(mut map) = self.0.lock() {
            let now = Utc::now();
            map.retain(|_, s| !s.expired(now));
            while map.len() >= MAX_STAGES {
                let oldest = map
                    .values()
                    .min_by_key(|s| s.created_at)
                    .map(|s| s.id.clone());
                match oldest {
                    Some(k) => {
                        map.remove(&k);
                    },
                    None => break,
                }
            }
            map.insert(id.clone(), stage);
        }
        id
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<StagedArchive> {
        let map = self.0.lock().ok()?;
        map.get(id).filter(|s| !s.expired(Utc::now())).cloned()
    }

    pub fn remove(&self, id: &str) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(id);
        }
    }

    // Why: an applied plane leaves the stage so the preview cannot apply it
    // twice; when the last one goes the stage goes with it.
    pub fn drop_plane(&self, id: &str, plane: &str) -> bool {
        let Ok(mut map) = self.0.lock() else {
            return false;
        };
        let Some(stage) = map.get_mut(id) else {
            return false;
        };
        stage.planes.remove(plane);
        if stage.planes.is_empty() {
            map.remove(id);
            return true;
        }
        false
    }
}
