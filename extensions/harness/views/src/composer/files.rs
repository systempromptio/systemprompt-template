//! The project's files offered for `@` mentions.
//!
//! A capped, gitignore-aware walk of the project directory that also skips
//! hidden directories and dependency and build output folders. It reads the
//! disk, so callers run it on the blocking pool.

use std::path::Path;

use ignore::{DirEntry, WalkBuilder};

pub const FILE_LIMIT: usize = 20_000;
const SKIPPED: [&str; 3] = ["node_modules", "target", "__pycache__"];

pub fn walk(root: &Path, limit: usize) -> Vec<String> {
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(is_walked)
        .build();
    let mut found = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::debug!(error = %error, "walk entry skipped");
                continue;
            },
        };
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        if let Some(relative) = relative(root, entry.path()) {
            found.push(relative);
        }
        if found.len() >= limit {
            break;
        }
    }
    found.sort();
    found
}

fn is_walked(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_some_and(|kind| kind.is_dir()) {
        return true;
    }
    let lossy = entry.file_name().to_string_lossy();
    let name: &str = &lossy;
    !name.starts_with('.') && !SKIPPED.contains(&name)
}

fn relative(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}
