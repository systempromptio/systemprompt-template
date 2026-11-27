use sp_drive::view::{DocumentItem, FolderNode};
use sp_protocol::ids::{DocumentFolderId, DocumentId};

pub(crate) fn id(n: u32) -> DocumentId {
    DocumentId::new(format!("00000000-0000-4000-8000-{n:012}"))
}

pub(crate) fn folder_id(n: u32) -> DocumentFolderId {
    DocumentFolderId::new(format!("10000000-0000-4000-8000-{n:012}"))
}

pub(crate) fn item(n: u32, name: &str, updated: &str, size: Option<u64>) -> DocumentItem {
    DocumentItem {
        id: id(n),
        name: name.to_owned(),
        format: "markdown".to_owned(),
        mime_type: "text/markdown".to_owned(),
        folder_id: None,
        updated_at: updated.to_owned(),
        size_bytes: size,
        my_access: None,
    }
}

pub(crate) fn node(n: u32, name: &str, children: Vec<FolderNode>) -> FolderNode {
    FolderNode {
        id: folder_id(n),
        name: name.to_owned(),
        agent_edit_policy: None,
        children,
    }
}
