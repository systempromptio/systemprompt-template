//! Helpers shared by the `extensions/web` integration tests.

use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    // Why: extensions/web sits two levels below the repo root.
    let mut root = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    for _ in 0..2 {
        root.pop();
    }
    root
}

pub(crate) fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}
