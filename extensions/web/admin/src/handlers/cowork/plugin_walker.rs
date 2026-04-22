use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct PluginFileEntry {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginEntry {
    pub id: String,
    pub version: String,
    pub sha256: String,
    pub files: Vec<PluginFileEntry>,
}

#[derive(serde::Deserialize)]
struct PluginMeta {
    version: Option<String>,
}

const BLOCKED: &[&str] = &["config.yaml", "config.yml"];

pub fn build_entry(
    plugins_root: &Path,
    plugin_id: &str,
    fallback_version: &str,
) -> Option<PluginEntry> {
    if !is_safe_id(plugin_id) {
        return None;
    }
    let plugin_dir = plugins_root.join(plugin_id);
    if !plugin_dir.is_dir() {
        return None;
    }
    let files = collect_files(&plugin_dir).ok()?;
    let dir_hash = hash_file_list(&files);
    let version = read_version(&plugin_dir).unwrap_or_else(|| fallback_version.to_string());
    Some(PluginEntry {
        id: plugin_id.to_string(),
        version,
        sha256: dir_hash,
        files,
    })
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && !id.contains("..")
        && !id.contains('/')
        && !id.contains('\\')
        && !id.starts_with('.')
}

fn collect_files(root: &Path) -> Result<Vec<PluginFileEntry>, std::io::Error> {
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk(
    base: &Path,
    dir: &Path,
    out: &mut Vec<PluginFileEntry>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let path = entry.path();
        if ft.is_dir() {
            walk(base, &path, out)?;
        } else if ft.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if BLOCKED.contains(&name) {
                    continue;
                }
            }
            let bytes = std::fs::read(&path)?;
            let mut h = Sha256::new();
            h.update(&bytes);
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(PluginFileEntry {
                path: rel,
                sha256: hex_encode(&h.finalize()),
                size: bytes.len() as u64,
            });
        }
    }
    Ok(())
}

fn hash_file_list(files: &[PluginFileEntry]) -> String {
    let mut hasher = Sha256::new();
    for f in files {
        hasher.update(f.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(f.sha256.as_bytes());
        hasher.update(b"\0");
    }
    hex_encode(&hasher.finalize())
}

fn read_version(plugin_dir: &Path) -> Option<String> {
    let candidates = [
        plugin_dir.join("claude-plugin").join("version.json"),
        plugin_dir.join("claude-plugin").join("plugin.json"),
        plugin_dir.join("plugin.json"),
    ];
    for p in &candidates {
        let bytes = std::fs::read(p).ok()?;
        let meta: PluginMeta = serde_json::from_slice(&bytes).ok()?;
        if let Some(v) = meta.version {
            return Some(v);
        }
    }
    None
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}
