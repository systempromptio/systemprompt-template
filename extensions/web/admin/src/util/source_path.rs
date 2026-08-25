//! Display form of a config file's path, as the catalog pages print it.

use std::path::Path;

// Why: `strip_prefix` returns Err when the path isn't under services_path
// (e.g. an absolute legacy entry); fall back to the full display path
// rather than bailing.
pub fn display_source_path(path: &Path, services_path: &Path) -> String {
    path.strip_prefix(services_path)
        .ok()
        .and_then(|p| p.to_str())
        .map_or_else(|| path.display().to_string(), |s| format!("services/{s}"))
}
