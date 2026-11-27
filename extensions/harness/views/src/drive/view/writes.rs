//! What the Drive screen writes: new folders and documents, comments, replies
//! and resolving a thread. After a write the parts it changed are read again.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{Context, Window};
use harness_ui::Tone;
use sp_drive::DocumentKind;
use sp_drive::view::DriveFailure;
use sp_protocol::ids::DocumentCommentId;
use sp_protocol::wire::drive::{CommentRequest, CreateDocumentRequest, DriveAnchor};

use super::{DriveView, jobs};
use crate::{bridge, strings as text};

impl DriveView {
    pub(super) fn create_folder(&mut self, name: &str, cx: &mut Context<Self>) {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return;
        }
        let Some(client) = self.client() else {
            return self.report(DriveFailure::SignedOut, cx);
        };
        let job = jobs::create_folder(client, name, self.browse.folder.clone());
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-create-folder",
            job,
            |this, result, cx| match result {
                Ok(folder) => {
                    if let Some(parent) = folder.parent_id {
                        this.browse.expanded.insert(parent);
                    }
                    this.notify_user(Tone::Success, text::FOLDER_CREATED, cx);
                    this.reload_folders(cx);
                },
                Err(error) => {
                    tracing::warn!(error = %error, "a Drive folder was not created");
                    this.report(DriveFailure::from(&error), cx);
                },
            },
        );
        self.keep(task);
    }

    pub(super) fn create_document(
        &mut self,
        kind: DocumentKind,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return;
        }
        let Some(client) = self.client() else {
            return self.report(DriveFailure::SignedOut, cx);
        };
        let request = CreateDocumentRequest {
            template_id: kind.template_id().to_owned(),
            name,
            folder_id: self.browse.folder.clone(),
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-create-document",
            jobs::create_document(client, request),
            |this, result, cx| match result {
                Ok(created) => {
                    this.notify_user(Tone::Success, text::DOCUMENT_CREATED, cx);
                    this.browse.selected = Some(created.document.id);
                    this.reload_listing(cx);
                },
                Err(error) => {
                    tracing::warn!(error = %error, "a Drive document was not created");
                    this.report(DriveFailure::from(&error), cx);
                },
            },
        );
        self.keep(task);
    }

    pub(super) fn reply_to(
        &mut self,
        comment: Option<DocumentCommentId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reply_to = comment;
        self.comment_field
            .update(cx, |field, cx| field.focus(window, cx));
        cx.notify();
    }

    pub(super) fn set_anchor(&mut self, node: String, cx: &mut Context<Self>) {
        self.anchor = Some(node);
        cx.notify();
    }

    pub(super) fn submit_comment(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let body = self.comment_field.read(cx).text(cx).trim().to_owned();
        if body.is_empty() {
            return self.notify_user(Tone::Warning, text::COMMENT_NEEDS_TEXT, cx);
        }
        let Some(id) = self.open.as_ref().map(|open| open.id.clone()) else {
            return;
        };
        let request = match (&self.reply_to, &self.anchor) {
            (Some(parent), _) => CommentRequest {
                body,
                anchor: None,
                parent_id: Some(parent.clone()),
                mention_options: Vec::new(),
            },
            (None, Some(node)) => CommentRequest {
                body,
                anchor: Some(DriveAnchor {
                    node: node.clone(),
                    find: None,
                }),
                parent_id: None,
                mention_options: Vec::new(),
            },
            (None, None) => return self.notify_user(Tone::Warning, text::COMMENT_NEEDS_ANCHOR, cx),
        };
        let Some(client) = self.client() else {
            return self.report(DriveFailure::SignedOut, cx);
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-comment",
            jobs::comment(client, id, request),
            |this, result, cx| match result {
                Ok(_) => {
                    this.reply_to = None;
                    this.clear_comment = true;
                    this.refresh_document(cx);
                },
                Err(error) => {
                    tracing::warn!(error = %error, "a Drive comment was not added");
                    this.report(DriveFailure::from(&error), cx);
                },
            },
        );
        self.keep(task);
    }

    pub(super) fn resolve_comment(&mut self, comment: DocumentCommentId, cx: &mut Context<Self>) {
        let Some(id) = self.open.as_ref().map(|open| open.id.clone()) else {
            return;
        };
        let Some(client) = self.client() else {
            return self.report(DriveFailure::SignedOut, cx);
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "drive-resolve-comment",
            jobs::resolve(client, id, comment),
            |this, result, cx| match result {
                Ok(_) => this.refresh_document(cx),
                Err(error) => {
                    tracing::warn!(error = %error, "a Drive comment was not resolved");
                    this.report(DriveFailure::from(&error), cx);
                },
            },
        );
        self.keep(task);
    }
}
