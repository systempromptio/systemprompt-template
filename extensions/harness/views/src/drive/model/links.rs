//! Where a document lives outside the app: its page on the instance and its
//! file under the local sync folder, refusing any path that leaves the folder.

use std::path::{Path, PathBuf};

use sp_protocol::ids::DocumentId;

pub fn web_link(instance_base: &str, id: Option<&DocumentId>) -> String {
    let base = instance_base.trim_end_matches('/');
    id.map_or_else(
        || format!("{base}/drive"),
        |id| format!("{base}/drive/{id}"),
    )
}

pub fn local_path(root: &Path, relative: &str) -> Option<PathBuf> {
    if relative.is_empty() || relative.starts_with('/') || relative.contains(['\\', ':']) {
        return None;
    }
    let mut path = root.to_path_buf();
    for segment in relative.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return None;
        }
        path.push(segment);
    }
    Some(path)
}
