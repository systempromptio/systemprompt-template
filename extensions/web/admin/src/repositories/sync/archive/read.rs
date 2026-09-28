//! Unpacks an uploaded zip under the rules a hostile archive would break.
//!
//! The rules mirror core's bundle extraction: no absolute or parent paths,
//! no symlinks, nothing outside `services/<allowed dir>/` but the manifest,
//! and every size checked as bytes are read rather than trusted from a
//! header — a zip bomb declares a small size and delivers a large one.
//! Everything read stays in memory; nothing here touches the disk.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use sha2::{Digest, Sha256};
use systemprompt::models::services::bundle::BUNDLE_ALLOWED_DIRS;

use super::manifest::{ArchiveManifest, parse_manifest};
use super::staging::{OtherEntry, StagedArchive};
use super::{MANIFEST_FILE, MAX_ENTRIES, MAX_ENTRY_BYTES, MAX_TOTAL_BYTES, TREE_PREFIX};
use crate::error::{AdminError, AdminResult};
use crate::repositories::sync::inventory::kind_for_path;
use crate::repositories::sync::plane::SyncPlane;

/// What an archive held once every rule passed.
#[derive(Debug, Default)]
pub struct Unpacked {
    pub manifest: Option<ArchiveManifest>,
    pub manifest_error: Option<String>,
    pub entries: BTreeMap<String, Vec<u8>>,
}

fn refuse(path: &str, why: &str) -> AdminError {
    AdminError::Unprocessable(format!("archive entry '{path}' refused: {why}"))
}

fn check_path(name: &str, enclosed: Option<&std::path::Path>) -> AdminResult<()> {
    // Why: `enclosed_name` relativises a leading `/` rather than rejecting
    // it, so the raw name is checked as well.
    if enclosed.is_none() || name.starts_with('/') {
        return Err(refuse(name, "absolute or parent path"));
    }
    if name.contains('\\') || name.contains('\0') {
        return Err(refuse(name, "backslash or NUL in the name"));
    }
    if name == MANIFEST_FILE {
        return Ok(());
    }
    let Some(rel) = name.strip_prefix(TREE_PREFIX) else {
        return Err(refuse(name, "outside services/"));
    };
    let top = rel.split('/').next().unwrap_or_default();
    if !BUNDLE_ALLOWED_DIRS.contains(&top) {
        return Err(refuse(name, "not a services directory the instance loads"));
    }
    Ok(())
}

pub fn unpack_zip(bytes: &[u8]) -> AdminResult<Unpacked> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    if archive.len() > MAX_ENTRIES {
        return Err(AdminError::Unprocessable(format!(
            "archive holds {} entries; the limit is {MAX_ENTRIES}",
            archive.len()
        )));
    }
    let mut out = Unpacked::default();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_owned();
        check_path(&name, file.enclosed_name().as_deref())?;
        if file.is_symlink() || file.unix_mode().is_some_and(|m| m & 0o170_000 == 0o120_000) {
            return Err(refuse(&name, "symlink"));
        }
        if file.size() > MAX_ENTRY_BYTES {
            return Err(refuse(&name, "larger than an entry may be"));
        }
        let mut buf = Vec::new();
        // Why: `take` one byte past the cap so an entry lying about its size
        // is caught by what it delivers, not what it declares.
        (&mut file)
            .take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut buf)
            .map_err(|e| refuse(&name, &e.to_string()))?;
        if buf.len() as u64 > MAX_ENTRY_BYTES {
            return Err(refuse(&name, "larger than an entry may be"));
        }
        total += buf.len() as u64;
        if total > MAX_TOTAL_BYTES {
            return Err(AdminError::Unprocessable(
                "archive expands past the size the instance accepts".to_owned(),
            ));
        }
        if name == MANIFEST_FILE {
            match String::from_utf8(buf)
                .map_err(|e| e.to_string())
                .and_then(|t| parse_manifest(&t))
            {
                Ok(m) => out.manifest = Some(m),
                Err(e) => out.manifest_error = Some(e),
            }
            continue;
        }
        out.entries.insert(name, buf);
    }
    Ok(out)
}

fn short_hash(bytes: &[u8]) -> String {
    let full = format!("{:x}", Sha256::digest(bytes));
    full.chars().take(12).collect()
}

// Why: every plane file must be text — it is parsed as its declaration —
// but any other entry is only described, so it may be anything.
pub fn classify(
    unpacked: Unpacked,
    registry: &[Box<dyn SyncPlane>],
    actor: &str,
) -> AdminResult<StagedArchive> {
    let mut staged = StagedArchive::new(actor, unpacked.manifest, unpacked.manifest_error);
    let mut entries = unpacked.entries;
    for plane in registry {
        let key = format!("{TREE_PREFIX}{}", plane.source_file());
        if let Some(bytes) = entries.remove(&key) {
            let text = String::from_utf8(bytes)
                .map_err(|e| AdminError::invalid("archive plane is not UTF-8 text", e))?;
            staged.planes.insert(plane.id(), text);
        }
    }
    for (path, bytes) in entries {
        let kind = kind_for_path(&path);
        staged.other.push(OtherEntry {
            hash: short_hash(&bytes),
            bytes: bytes.len(),
            kind_id: kind.map(|k| k.id),
            kind_label: kind.map(|k| k.label),
            path,
        });
    }
    Ok(staged)
}
