//! The Drive screen.
//!
//! It shows the documents and folders of the instance the Drive handle
//! follows, a preview of the chosen document with its comments, revisions and
//! sharing, and the local folder the sync keeps. While the handle is withheld
//! the screen says so and holds nothing of the last instance. Every Drive call
//! runs on the runtime through [`bridge`](crate::bridge); the view only applies
//! results. It follows the instance's enablement, which changes after every
//! apply, to notice when the handle moves to another account or instance.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod browse;
mod comments;
mod dialogs;
mod documents;
mod imports;
mod intents;
mod jobs;
mod preview;
mod preview_parts;
mod render;
mod sync_details;
mod sync_panel;
mod writes;

use gpui_kit::{AppContext as _, Context, Entity, FocusHandle, Task, Window};
use harness_shell::ShellContext;
use harness_tasks::CancellationToken;
use harness_ui::components::TextField;
use harness_ui::{MarkdownDocument, Tone};
use sp_accounts::Accounts;
use sp_drive::view::{CommentThread, DocumentItem, DriveFailure, RevisionItem, ShareState};
use sp_drive::{DriveClient, DriveHandle, DriveStatus, UploadProgress};
use sp_protocol::ids::{DocumentCommentId, DocumentId};
use sp_protocol::wire::drive::DriveDetail;
use tokio::runtime::Handle;
use tokio::sync::watch;

use crate::model::{Browse, DriveTarget, Load, PreviewText};
use crate::{bridge, strings as text};

const FOLDER_PANE: f32 = 232.0;
const LIST_PANE: f32 = 360.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PreviewTab {
    #[default]
    Preview,
    Comments,
    Revisions,
    Sharing,
}

impl PreviewTab {
    const ALL: [Self; 4] = [
        Self::Preview,
        Self::Comments,
        Self::Revisions,
        Self::Sharing,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Preview => text::TAB_PREVIEW,
            Self::Comments => text::TAB_COMMENTS,
            Self::Revisions => text::TAB_REVISIONS,
            Self::Sharing => text::TAB_SHARING,
        }
    }
}

struct OpenDocument {
    id: DocumentId,
    item: DocumentItem,
    detail: Load<DriveDetail>,
    preview: Load<PreviewText>,
    markdown: MarkdownDocument,
    comments: Load<Vec<CommentThread>>,
    revisions: Load<Vec<RevisionItem>>,
    access: Load<ShareState>,
}

impl OpenDocument {
    fn loading(item: DocumentItem) -> Self {
        Self {
            id: item.id.clone(),
            item,
            detail: Load::Loading,
            preview: Load::Loading,
            markdown: MarkdownDocument::new(),
            comments: Load::Loading,
            revisions: Load::Loading,
            access: Load::Loading,
        }
    }
}

struct Upload {
    name: String,
    progress: watch::Receiver<Option<UploadProgress>>,
    cancel: CancellationToken,
    _job: Task<()>,
    _feed: Task<()>,
}

#[derive(Default)]
struct Tasks {
    listing: Option<Task<()>>,
    folders: Option<Task<()>>,
    document: Option<Task<()>>,
    search: Option<Task<()>>,
    side: Vec<Task<()>>,
}

pub(crate) struct DriveView {
    handle: DriveHandle,
    accounts: Accounts,
    runtime: Handle,
    sync_status: watch::Receiver<DriveStatus>,
    sync_details: bool,
    target: Option<DriveTarget>,
    browse: Browse,
    open: Option<OpenDocument>,
    tab: PreviewTab,
    upload: Option<Upload>,
    search: Entity<TextField>,
    comment_field: Entity<TextField>,
    reply_to: Option<DocumentCommentId>,
    anchor: Option<String>,
    clear_comment: bool,
    notices: Vec<(Tone, &'static str)>,
    list_focus: FocusHandle,
    tasks: Tasks,
    _follow: [Task<()>; 2],
}

impl DriveView {
    pub(crate) fn new(
        handle: DriveHandle,
        ctx: &ShellContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let weak = cx.weak_entity();
        let search = cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(text::SEARCH)
                .on_change(move |query, _, cx| {
                    let query = query.to_string();
                    if weak
                        .update(cx, |this, cx| this.set_query(query, cx))
                        .is_err()
                    {
                        tracing::trace!("search changed after the Drive closed");
                    }
                })
        });
        let weak = cx.weak_entity();
        let comment_field = cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(text::COMMENT_PLACEHOLDER)
                .on_submit(move |_, window, cx| {
                    if weak
                        .update(cx, |this, cx| this.submit_comment(window, cx))
                        .is_err()
                    {
                        tracing::trace!("comment submitted after the Drive closed");
                    }
                })
        });
        let sync_feed = handle.sync().subscribe();
        let follow = [
            bridge::follow(cx, ctx.enablement.subscribe(), Self::follow_target),
            bridge::follow(cx, handle.sync().subscribe(), |_: &mut Self, cx| {
                cx.notify();
            }),
        ];
        let mut view = Self {
            handle,
            accounts: ctx.accounts.clone(),
            runtime: ctx.runtime.clone(),
            sync_status: sync_feed,
            sync_details: false,
            target: None,
            browse: Browse::default(),
            open: None,
            tab: PreviewTab::default(),
            upload: None,
            search,
            comment_field,
            reply_to: None,
            anchor: None,
            clear_comment: false,
            notices: Vec::new(),
            list_focus: cx.focus_handle(),
            tasks: Tasks::default(),
            _follow: follow,
        };
        view.follow_target(cx);
        view
    }

    fn current_target(&self) -> Option<DriveTarget> {
        let client = self.handle.client().ok()?;
        Some(DriveTarget::new(
            self.accounts.snapshot().active,
            client.base(),
        ))
    }

    fn follow_target(&mut self, cx: &mut Context<Self>) {
        let target = self.current_target();
        if target == self.target {
            cx.notify();
            return;
        }
        self.target = target;
        self.browse.reset();
        self.open = None;
        self.reply_to = None;
        self.anchor = None;
        if let Some(upload) = self.upload.take() {
            upload.cancel.cancel();
        }
        if self.target.is_some() {
            self.reload_all(cx);
        }
        cx.notify();
    }

    fn client(&self) -> Option<DriveClient> {
        match self.handle.client() {
            Ok(client) => Some(client),
            Err(withheld) => {
                tracing::debug!(error = %withheld, "the Drive has no client");
                None
            },
        }
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.side.retain(|task| !task.is_ready());
        self.tasks.side.push(task);
    }

    fn notify_user(&mut self, tone: Tone, message: &'static str, cx: &mut Context<Self>) {
        self.notices.push((tone, message));
        cx.notify();
    }

    fn report(&mut self, failure: DriveFailure, cx: &mut Context<Self>) {
        self.notify_user(Tone::Danger, text::failure(failure), cx);
    }
}

impl std::fmt::Debug for DriveView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveView")
            .field("target", &self.target)
            .field("open", &self.open.as_ref().map(|open| &open.id))
            .finish_non_exhaustive()
    }
}
