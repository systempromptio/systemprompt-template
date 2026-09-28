//! The `base` source's content hash: this repository's `services/` tree
//! hashed exactly as `systemprompt core services bundle` hashes a bundle.
//!
//! Same file walk (`collect_files` over `BUNDLE_ALLOWED_DIRS`), same digest
//! (`compute_content_hash`), so the number on the sync page equals the
//! `content_hash` of the base bundle `release.yml` publishes at this release
//! — one algorithm, one hash for "the internal services at this version".

use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use sha2::{Digest, Sha256};
use systemprompt::loader::bundle::pack::collect_files;
use systemprompt::models::services::bundle::{
    BUNDLE_ALLOWED_DIRS, FileEntry, ServicesBundleManifest,
};

// Why: hashing the tree reads every file, so the result is cached per
// process and recomputed only when the tree's newest mtime or file count
// moves — a `just publish` or a deploy, not a page view.
pub(super) type Fingerprint = (SystemTime, usize);
type Cached = Option<(Fingerprint, String)>;

pub fn base_tree_hash(root: &Path) -> Option<String> {
    static CACHE: OnceLock<Mutex<Cached>> = OnceLock::new();
    let fingerprint = tree_fingerprint(root, BUNDLE_ALLOWED_DIRS)?;
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock()
        && let Some((fp, hash)) = guard.as_ref()
        && *fp == fingerprint
    {
        return Some(hash.clone());
    }
    let files = collect_files(root, BUNDLE_ALLOWED_DIRS).ok()?;
    let hash = ServicesBundleManifest::compute_content_hash(&files);
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((fingerprint, hash.clone()));
    }
    Some(hash)
}

/// One kind's hash: the same digest over just its file or directory.
#[derive(Debug, Clone)]
pub struct SubtreeHash {
    pub hash: String,
    pub files: usize,
}

// Why: the same walk and digest as the whole tree, narrowed to one path, so
// a kind's hash on the configuration page is comparable with the hash a
// bundle carrying only that kind would declare.
#[must_use]
pub fn subtree_hash(root: &Path, rel: &str, is_dir: bool) -> Option<SubtreeHash> {
    let files = if is_dir {
        collect_files(root, &[rel]).ok()?
    } else {
        let content = std::fs::read(root.join(rel)).ok()?;
        vec![FileEntry {
            path: rel.to_owned(),
            sha256: hex::encode(Sha256::digest(&content)),
            size: content.len() as u64,
        }]
    };
    (!files.is_empty()).then(|| SubtreeHash {
        hash: ServicesBundleManifest::compute_content_hash(&files),
        files: files.len(),
    })
}

pub(super) fn tree_fingerprint(root: &Path, dirs: &[&str]) -> Option<Fingerprint> {
    let mut newest = SystemTime::UNIX_EPOCH;
    let mut count = 0usize;
    for dir in dirs {
        walk(&root.join(dir), &mut newest, &mut count);
    }
    (count > 0).then_some((newest, count))
}

fn walk(dir: &Path, newest: &mut SystemTime, count: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, newest, count);
        } else if let Ok(meta) = entry.metadata() {
            *count += 1;
            if let Ok(m) = meta.modified()
                && m > *newest
            {
                *newest = m;
            }
        }
    }
}
