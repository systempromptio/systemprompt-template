//! Repository-root discovery for tests that read the source tree.
//!
//! The root is found by climbing until the workspace markers appear, never by
//! a fixed ancestor depth: a stale depth points at a directory that holds
//! nothing, every walk over it finds nothing, and the assertions built on it
//! pass having examined no files. Callers get a path or a panic, never a
//! `None` they can quietly return on.

use std::path::{Path, PathBuf};

#[must_use]
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|dir| dir.join("services").is_dir() && dir.join("extensions").is_dir())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            panic!(
                "no repository root above {}: expected an ancestor holding both services/ and \
                 extensions/",
                env!("CARGO_MANIFEST_DIR")
            )
        })
}

#[must_use]
pub fn repo_path(relative: &str) -> PathBuf {
    let path = repo_root().join(relative);
    assert!(
        path.exists(),
        "{} does not exist under the repository root",
        path.display()
    );
    path
}
