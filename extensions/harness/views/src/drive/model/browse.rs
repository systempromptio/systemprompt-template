//! What the Drive screen browses: the scope, folder, search and sort, the page
//! of documents with its cursor, the selection, and the generation that drops
//! a listing answered after a newer one was asked for.

use std::collections::HashSet;

use sp_drive::ListQuery;
use sp_drive::view::{DocumentItem, DriveFailure, FolderNode};
use sp_protocol::ids::{DocumentFolderId, DocumentId};
use sp_protocol::wire::drive::DriveScope;

const PAGE_SIZE: u32 = 100;

/// One read of the Drive as the screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Load<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(DriveFailure),
}

impl<T> Load<T> {
    pub const fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
}

/// Whose documents the list shows when no folder is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseScope {
    #[default]
    Mine,
    Shared,
    Recent,
}

impl BrowseScope {
    pub const ALL: [Self; 3] = [Self::Mine, Self::Shared, Self::Recent];

    pub const fn wire(self) -> DriveScope {
        match self {
            Self::Mine => DriveScope::Mine,
            Self::Shared => DriveScope::Shared,
            Self::Recent => DriveScope::Recent,
        }
    }
}

/// The order of the document list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    Updated,
    Name,
    Size,
}

impl SortKey {
    pub const ALL: [Self; 3] = [Self::Updated, Self::Name, Self::Size];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Name => "name",
            Self::Size => "size",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sort| sort.key() == key)
    }
}

pub fn sort_documents(items: &mut [DocumentItem], sort: SortKey) {
    match sort {
        SortKey::Updated => items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
        SortKey::Name => items.sort_by_key(|item| item.name.to_lowercase()),
        SortKey::Size => items.sort_by_key(|a| std::cmp::Reverse(a.size_bytes.unwrap_or(0))),
    }
}

/// The browsing state of the Drive screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Browse {
    pub scope: BrowseScope,
    pub folder: Option<DocumentFolderId>,
    pub query: String,
    pub sort: SortKey,
    pub folders: Load<Vec<FolderNode>>,
    pub expanded: HashSet<DocumentFolderId>,
    pub documents: Load<Vec<DocumentItem>>,
    pub next_cursor: Option<String>,
    pub selected: Option<DocumentId>,
    pub generation: u64,
}

impl Browse {
    pub fn list_query(&self, cursor: Option<String>) -> ListQuery {
        let query = Some(self.query.trim().to_owned()).filter(|text| !text.is_empty());
        let scope = if self.folder.is_some() {
            DriveScope::Folder
        } else {
            self.scope.wire()
        };
        ListQuery {
            scope,
            folder: self.folder.clone(),
            query,
            sort: None,
            limit: Some(PAGE_SIZE),
            cursor,
        }
    }

    pub fn begin(&mut self, appending: bool) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        if !appending {
            self.documents = Load::Loading;
            self.next_cursor = None;
        }
        self.generation
    }

    pub fn apply_page(
        &mut self,
        generation: u64,
        items: Vec<DocumentItem>,
        cursor: Option<String>,
    ) -> bool {
        if generation != self.generation {
            return false;
        }
        if let Load::Ready(known) = &mut self.documents {
            for item in items {
                if !known.iter().any(|existing| existing.id == item.id) {
                    known.push(item);
                }
            }
        } else {
            self.documents = Load::Ready(items);
        }
        if let Load::Ready(known) = &mut self.documents {
            sort_documents(known, self.sort);
        }
        self.next_cursor = cursor;
        true
    }

    pub fn fail_page(&mut self, generation: u64, failure: DriveFailure) -> bool {
        if generation != self.generation {
            return false;
        }
        self.documents = Load::Failed(failure);
        true
    }

    pub fn set_sort(&mut self, sort: SortKey) {
        self.sort = sort;
        if let Load::Ready(items) = &mut self.documents {
            sort_documents(items, sort);
        }
    }

    pub fn item(&self, id: &DocumentId) -> Option<&DocumentItem> {
        self.documents.ready()?.iter().find(|item| &item.id == id)
    }

    pub fn step(&self, forward: bool) -> Option<DocumentId> {
        let items = self.documents.ready()?;
        let current = self
            .selected
            .as_ref()
            .and_then(|id| items.iter().position(|item| &item.id == id));
        let next = match (current, forward) {
            (None, true) => 0,
            (None, false) => items.len().checked_sub(1)?,
            (Some(index), true) => (index + 1).min(items.len().checked_sub(1)?),
            (Some(index), false) => index.saturating_sub(1),
        };
        items.get(next).map(|item| item.id.clone())
    }

    pub fn toggle_folder(&mut self, id: &DocumentFolderId) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.clone());
        }
    }

    pub fn reset(&mut self) {
        *self = Self {
            generation: self.generation.wrapping_add(1),
            ..Self::default()
        };
    }
}
