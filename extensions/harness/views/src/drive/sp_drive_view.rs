//! What the shell draws from the Drive: document rows, the folder tree, comment
//! threads and failure kinds.
//!
//! Revisions, proposals, access and grants are drawn from the protocol types
//! themselves, re-exported here.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::{BTreeMap, BTreeSet};

use sp_client::ClientError;
use sp_protocol::ids::{DocumentFolderId, DocumentId};
use sp_protocol::wire::drive::{DriveComment, DriveDocument, DriveFolder};

use crate::error::{DriveError, NeededCapability};

pub use sp_protocol::wire::drive::{
    DriveAccess as ShareState, DriveGrant as ShareGrant, DriveProposal as ProposalItem,
    DriveRevision as RevisionItem,
};

/// A row of a document listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentItem {
    pub id: DocumentId,
    pub name: String,
    pub format: String,
    pub mime_type: String,
    pub folder_id: Option<DocumentFolderId>,
    pub updated_at: String,
    pub size_bytes: Option<u64>,
    pub my_access: Option<String>,
}

impl From<&DriveDocument> for DocumentItem {
    fn from(document: &DriveDocument) -> Self {
        Self {
            id: document.id.clone(),
            name: document.name.clone(),
            format: document.format.clone(),
            mime_type: document.mime_type.clone(),
            folder_id: document.folder_id.clone(),
            updated_at: document.updated_at.clone(),
            size_bytes: document.size_bytes,
            my_access: document.my_access.clone(),
        }
    }
}

/// A folder with the folders under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderNode {
    pub id: DocumentFolderId,
    pub name: String,
    pub agent_edit_policy: Option<String>,
    pub children: Vec<Self>,
}

/// A top-level comment and its replies, oldest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentThread {
    pub root: DriveComment,
    pub replies: Vec<DriveComment>,
}

impl CommentThread {
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        self.root.resolved_at.is_some()
    }
}

/// Why a Drive call failed, in the words the shell picks its message by. Only
/// `Unreachable` is worth retrying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveFailure {
    SignedOut,
    Unreachable,
    NotFound,
    Forbidden,
    PersonOnly,
    SuggestOnly,
    Conflict,
    Cancelled,
    Invalid,
    Failed,
}

impl From<&DriveError> for DriveFailure {
    fn from(error: &DriveError) -> Self {
        match error {
            DriveError::Client(client) => match client {
                ClientError::Unauthorized { .. }
                | ClientError::SessionMismatch { .. }
                | ClientError::Credential(_) => Self::SignedOut,
                ClientError::Unreachable { .. } | ClientError::RateLimited { .. } => {
                    Self::Unreachable
                },
                ClientError::InvalidInput { .. } => Self::Invalid,
                _ => Self::Failed,
            },
            DriveError::NotFound { .. } => Self::NotFound,
            DriveError::AgentRefused {
                capability: NeededCapability::Edit,
                ..
            } => Self::SuggestOnly,
            DriveError::AgentRefused {
                capability: NeededCapability::Person,
                ..
            } => Self::PersonOnly,
            DriveError::Forbidden { .. } | DriveError::AgentRefused { .. } => Self::Forbidden,
            DriveError::Conflict { .. } => Self::Conflict,
            DriveError::Cancelled => Self::Cancelled,
            DriveError::InvalidInput { .. } => Self::Invalid,
            DriveError::PartAltered { .. } => Self::Failed,
        }
    }
}

#[must_use]
pub fn document_items(documents: &[DriveDocument]) -> Vec<DocumentItem> {
    documents.iter().map(DocumentItem::from).collect()
}

#[must_use]
pub fn folder_tree(folders: &[DriveFolder]) -> Vec<FolderNode> {
    let known: BTreeSet<String> = folders.iter().map(|folder| folder.id.to_string()).collect();
    let mut children: BTreeMap<String, Vec<&DriveFolder>> = BTreeMap::new();
    let mut roots: Vec<&DriveFolder> = Vec::new();
    for folder in folders {
        match &folder.parent_id {
            Some(parent) if known.contains(&parent.to_string()) && *parent != folder.id => {
                children.entry(parent.to_string()).or_default().push(folder);
            },
            _ => roots.push(folder),
        }
    }
    let mut placed = BTreeSet::new();
    let mut tree: Vec<FolderNode> = roots
        .into_iter()
        .map(|folder| node(folder, &children, &mut placed))
        .collect();
    let orphans: Vec<&DriveFolder> = folders
        .iter()
        .filter(|folder| !placed.contains(&folder.id.to_string()))
        .collect();
    for folder in orphans {
        if !placed.contains(&folder.id.to_string()) {
            tree.push(node(folder, &children, &mut placed));
        }
    }
    sort_nodes(&mut tree);
    tree
}

#[must_use]
pub fn comment_threads(comments: Vec<DriveComment>) -> Vec<CommentThread> {
    let ids: BTreeSet<String> = comments
        .iter()
        .map(|comment| comment.id.to_string())
        .collect();
    let mut threads: Vec<CommentThread> = Vec::new();
    let mut replies: BTreeMap<String, Vec<DriveComment>> = BTreeMap::new();
    for comment in comments {
        match comment.parent_id.as_ref().map(ToString::to_string) {
            Some(parent) if ids.contains(&parent) => {
                replies.entry(parent).or_default().push(comment);
            },
            _ => threads.push(CommentThread {
                root: comment,
                replies: Vec::new(),
            }),
        }
    }
    for thread in &mut threads {
        if let Some(mut found) = replies.remove(&thread.root.id.to_string()) {
            found.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            thread.replies = found;
        }
    }
    threads.sort_by(|a, b| a.root.created_at.cmp(&b.root.created_at));
    threads
}

fn node(
    folder: &DriveFolder,
    children: &BTreeMap<String, Vec<&DriveFolder>>,
    placed: &mut BTreeSet<String>,
) -> FolderNode {
    let key = folder.id.to_string();
    placed.insert(key.clone());
    let mut kids = Vec::new();
    for child in children.get(&key).map(Vec::as_slice).unwrap_or_default() {
        if !placed.contains(&child.id.to_string()) {
            kids.push(node(child, children, placed));
        }
    }
    FolderNode {
        id: folder.id.clone(),
        name: folder.name.clone(),
        agent_edit_policy: folder.agent_edit_policy.clone(),
        children: kids,
    }
}

fn sort_nodes(nodes: &mut [FolderNode]) {
    nodes.sort_by_key(|node| node.name.to_lowercase());
    for node in nodes {
        sort_nodes(&mut node.children);
    }
}
