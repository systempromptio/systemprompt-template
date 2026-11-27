//! What the Drive screen does when the user browses: reload the tree and the
//! listing, switch scope, folder, search and sort, move the selection, open a
//! document and its tabs, open it in the browser or hand it to the composer.

use std::time::Duration;

use gpui_kit::{AsyncApp, Context, SharedString, WeakEntity, Window};
use harness_shell::DraftInSession;
use harness_ui::MarkdownDocument;
use sp_drive::DriveError;
use sp_drive::view::DriveFailure;
use sp_protocol::ids::{DocumentFolderId, DocumentId};

use super::jobs::{self, DocumentLoad};
use super::{DriveView, OpenDocument, PreviewTab};
use crate::model::{BrowseScope, Load, SortKey, web_link};
use crate::{bridge, strings as text};

const SEARCH_PAUSE: Duration = Duration::from_millis(250);

pub(super) fn settle<T>(result: Result<T, DriveError>, what: &'static str) -> Load<T> {
    match result {
        Ok(value) => Load::Ready(value),
        Err(error) => {
            tracing::warn!(what, error = %error, "a Drive read failed");
            Load::Failed(DriveFailure::from(&error))
        },
    }
}

impl DriveView {
    pub(super) fn reload_all(&mut self, cx: &mut Context<Self>) {
        self.reload_folders(cx);
        self.reload_listing(cx);
        if let Some(id) = self.browse.selected.clone() {
            self.load_document(id, cx);
        }
    }

    pub(super) fn reload_folders(&mut self, cx: &Context<Self>) {
        let Some(client) = self.client() else {
            self.browse.folders = Load::Failed(DriveFailure::SignedOut);
            return;
        };
        if self.browse.folders.ready().is_none() {
            self.browse.folders = Load::Loading;
        }
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-folders",
            jobs::folders(client),
            |this, result, cx| {
                this.browse.folders = settle(result, "folders");
                cx.notify();
            },
        );
        self.tasks.folders = Some(task);
    }

    pub(super) fn reload_listing(&mut self, cx: &mut Context<Self>) {
        self.fetch_page(None, cx);
    }

    pub(super) fn load_more(&mut self, cx: &mut Context<Self>) {
        if let Some(cursor) = self.browse.next_cursor.clone() {
            self.fetch_page(Some(cursor), cx);
        }
    }

    fn fetch_page(&mut self, cursor: Option<String>, cx: &mut Context<Self>) {
        let appending = cursor.is_some();
        let generation = self.browse.begin(appending);
        let Some(client) = self.client() else {
            self.browse.fail_page(generation, DriveFailure::SignedOut);
            cx.notify();
            return;
        };
        let query = self.browse.list_query(cursor);
        let job = jobs::page(client, query);
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-listing",
            job,
            move |this, result, cx| {
                let applied = match result {
                    Ok(page) => this
                        .browse
                        .apply_page(generation, page.items, page.next_cursor),
                    Err(error) => {
                        tracing::warn!(error = %error, "a Drive listing failed");
                        this.browse
                            .fail_page(generation, DriveFailure::from(&error))
                    },
                };
                if applied {
                    cx.notify();
                }
            },
        );
        self.tasks.listing = Some(task);
        cx.notify();
    }

    pub(super) fn set_scope(&mut self, scope: BrowseScope, cx: &mut Context<Self>) {
        if self.browse.scope == scope && self.browse.folder.is_none() {
            return;
        }
        self.browse.scope = scope;
        self.browse.folder = None;
        self.reload_listing(cx);
    }

    pub(super) fn select_folder(
        &mut self,
        folder: Option<DocumentFolderId>,
        cx: &mut Context<Self>,
    ) {
        if self.browse.folder == folder {
            return;
        }
        self.browse.folder = folder;
        self.reload_listing(cx);
    }

    pub(super) fn toggle_folder(&mut self, folder: &DocumentFolderId, cx: &mut Context<Self>) {
        self.browse.toggle_folder(folder);
        cx.notify();
    }

    pub(super) fn set_query(&mut self, query: String, cx: &Context<Self>) {
        if self.browse.query == query {
            return;
        }
        self.browse.query = query;
        let task = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            cx.background_executor().timer(SEARCH_PAUSE).await;
            if this.update(cx, Self::reload_listing).is_err() {
                tracing::trace!("search settled after the Drive closed");
            }
        });
        self.tasks.search = Some(task);
    }

    pub(super) fn set_sort(&mut self, sort: SortKey, cx: &mut Context<Self>) {
        self.browse.set_sort(sort);
        cx.notify();
    }

    pub(super) fn step_selection(&mut self, forward: bool, cx: &mut Context<Self>) {
        if let Some(id) = self.browse.step(forward) {
            self.select_document(id, cx);
        }
    }

    pub(super) fn select_document(&mut self, id: DocumentId, cx: &mut Context<Self>) {
        if self.open.as_ref().is_some_and(|open| open.id == id) {
            return;
        }
        self.browse.selected = Some(id.clone());
        self.reply_to = None;
        self.anchor = None;
        self.load_document(id, cx);
    }

    pub(super) fn load_document(&mut self, id: DocumentId, cx: &mut Context<Self>) {
        let Some(item) = self.browse.item(&id).cloned() else {
            return;
        };
        let Some(client) = self.client() else { return };
        let keep_body = self.open.as_ref().is_some_and(|open| open.id == id);
        if !keep_body {
            self.open = Some(OpenDocument::loading(item));
        }
        let job = jobs::document(client, id.clone());
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-document",
            job,
            move |this, load, cx| {
                this.document_loaded(&id, load);
                cx.notify();
            },
        );
        self.tasks.document = Some(task);
        cx.notify();
    }

    fn document_loaded(&mut self, id: &DocumentId, load: DocumentLoad) {
        let Some(open) = self.open.as_mut().filter(|open| &open.id == id) else {
            return;
        };
        open.detail = settle(load.detail, "document");
        open.preview = settle(load.preview, "preview");
        if let Load::Ready(preview) = &open.preview {
            open.markdown = MarkdownDocument::from_text(&preview.markdown);
            let known = self
                .anchor
                .as_ref()
                .is_some_and(|node| preview.anchors.iter().any(|anchor| &anchor.node == node));
            if !known {
                self.anchor = preview.anchors.first().map(|anchor| anchor.node.clone());
            }
        }
        open.comments = settle(load.comments, "comments");
        open.revisions = settle(load.revisions, "revisions");
        open.access = settle(load.access, "access");
    }

    pub(super) fn refresh_document(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.open.as_ref().map(|open| open.id.clone()) {
            self.load_document(id, cx);
        }
    }

    pub(super) fn set_tab(&mut self, tab: PreviewTab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }

    pub(super) fn open_in_browser(&self, id: Option<&DocumentId>, cx: &Context<Self>) {
        if let Some(target) = &self.target {
            cx.open_url(&web_link(target.instance_base(), id));
        }
    }

    pub(super) fn attach_to_composer(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(open) = &self.open else { return };
        let text = text::attach_reference(&open.item.name, open.id.as_str());
        window.dispatch_action(
            Box::new(DraftInSession {
                text: SharedString::from(text),
            }),
            cx,
        );
    }
}
