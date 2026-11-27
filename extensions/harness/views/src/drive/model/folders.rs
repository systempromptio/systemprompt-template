//! The folder tree flattened into the rows the folder pane draws, following
//! which folders are expanded.

use std::collections::HashSet;
use std::hash::BuildHasher;

use sp_drive::view::FolderNode;
use sp_protocol::ids::DocumentFolderId;

/// One visible folder: its depth in the tree and whether it can open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    pub id: DocumentFolderId,
    pub name: String,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

pub fn folder_rows<S: BuildHasher>(
    tree: &[FolderNode],
    expanded: &HashSet<DocumentFolderId, S>,
) -> Vec<FolderRow> {
    let mut rows = Vec::new();
    push_rows(tree, expanded, 0, &mut rows);
    rows
}

fn push_rows<S: BuildHasher>(
    nodes: &[FolderNode],
    expanded: &HashSet<DocumentFolderId, S>,
    depth: usize,
    rows: &mut Vec<FolderRow>,
) {
    for node in nodes {
        let open = expanded.contains(&node.id);
        rows.push(FolderRow {
            id: node.id.clone(),
            name: node.name.clone(),
            depth,
            has_children: !node.children.is_empty(),
            expanded: open,
        });
        if open {
            push_rows(&node.children, expanded, depth + 1, rows);
        }
    }
}
