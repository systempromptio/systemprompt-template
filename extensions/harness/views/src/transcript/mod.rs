//! The transcript of one session.
//!
//! A virtualised list of its messages with streamed markdown, collapsible
//! reasoning, one compact row per tool call, diffs, todos and subagent
//! progress, the session header with its changes panel, and the status lines
//! under the conversation.
//!
//! Contract (fixed by the shell): `TranscriptView::new(session, app, services,
//! window, cx)` is built once per open session by the frame. It renders from
//! the session's `SessionModel` and acts only through `SessionState`'s intents.
//! Opening a subagent session and handing a reverted prompt back to the
//! composer are emitted as `TranscriptEvent`s for the frame to route.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod actions;
mod adornments;
mod cache;
mod changes;
pub mod children;
mod footer;
mod message;
mod message_actions;
mod parts;
pub mod rows;
mod survey;
mod tool_body;
pub mod tool_info;
mod tool_row;

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::{
    AsyncApp, Context, Entity, EventEmitter, FocusHandle, Focusable, FollowMode,
    InteractiveElement as _, IntoElement, ListAlignment, ListScrollEvent, ListState,
    ParentElement as _, Render, Styled as _, Subscription, Task, WeakEntity, Window, div, px,
};
use harness_engine::RunStatus;
use harness_store::{Change, PartId, SessionId};
use harness_ui::components::LIST_CONTEXT;
use harness_ui::theme::tokens;

use crate::app::Services;
use crate::app::runtime::{self, FRAME};
use crate::state::extensions::Contributions;
use crate::state::{AppState, SessionState};
use crate::views::transcript::cache::{DocumentCache, PatchCache};
use crate::views::transcript::children::Children;
use crate::views::transcript::footer::RetryClock;
use crate::views::transcript::rows::RowKey;
use crate::views::transcript::survey::survey;

const OVERDRAW: f32 = 800.0;
const TICK: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TranscriptEvent {
    OpenSession(SessionId),
    RestoreDraft(String),
}

pub(crate) struct TranscriptView {
    session: Entity<SessionState>,
    app: Entity<AppState>,
    adornments_from: Option<Rc<Contributions>>,
    adornment_follows: Vec<Task<()>>,
    services: Services,
    list: ListState,
    rows: Vec<RowKey>,
    revision: Option<u64>,
    documents: DocumentCache,
    patches: PatchCache<PartId>,
    file_patches: PatchCache<PathBuf>,
    expanded: HashSet<PartId>,
    selected: Option<usize>,
    following: bool,
    changes_open: bool,
    children: Children,
    retry: RetryClock,
    focus: FocusHandle,
    ticker: Option<Task<()>>,
    loads: Vec<Task<()>>,
    _observations: [Subscription; 2],
    _feed: Task<()>,
}

impl std::fmt::Debug for TranscriptView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranscriptView")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

impl EventEmitter<TranscriptEvent> for TranscriptView {}

impl TranscriptView {
    pub(crate) fn new(
        session: Entity<SessionState>,
        app: Entity<AppState>,
        services: Services,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let list = ListState::new(0, ListAlignment::Top, px(OVERDRAW));
        list.set_follow_mode(FollowMode::Tail);
        list.set_scroll_handler(
            cx.listener(|this: &mut Self, event: &ListScrollEvent, _, cx| {
                if this.following != event.is_following_tail {
                    this.following = event.is_following_tail;
                    cx.notify();
                }
            }),
        );
        let observations = [
            cx.observe(&session, |this, _, cx| this.sync(cx)),
            cx.observe(&app, |this, app, cx| this.follow_adornments(&app, cx)),
        ];
        let feed = runtime::pump_batched(
            cx,
            services.store().subscribe(),
            FRAME,
            Self::on_changes,
            |_, _, _| {},
        );
        let mut view = Self {
            session,
            app,
            adornments_from: None,
            adornment_follows: Vec::new(),
            services,
            list,
            rows: Vec::new(),
            revision: None,
            documents: DocumentCache::default(),
            patches: PatchCache::default(),
            file_patches: PatchCache::default(),
            expanded: HashSet::new(),
            selected: None,
            following: true,
            changes_open: false,
            children: Children::default(),
            retry: RetryClock::default(),
            focus: cx.focus_handle(),
            ticker: None,
            loads: Vec::new(),
            _observations: observations,
            _feed: feed,
        };
        let app = view.app.clone();
        view.follow_adornments(&app, cx);
        view.sync(cx);
        view
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let survey = survey(self.session.read(cx).model());
        if self.revision == Some(survey.revision) {
            return;
        }
        self.revision = Some(survey.revision);
        if let Some((range, count)) = rows::changed_range(&self.rows, &survey.rows) {
            self.list.splice(range, count);
        }
        self.rows = survey.rows;
        self.selected = self.selected.filter(|selected| *selected < self.rows.len());
        self.retry.observe(&survey.status);
        self.update_ticker(matches!(survey.status, RunStatus::Retry { .. }), cx);
        self.documents.retain(&survey.live_parts);
        self.patches.retain(|id| survey.live_parts.contains(id));
        self.expanded.retain(|id| survey.live_parts.contains(id));
        for child in survey.children {
            self.watch_child(child, cx);
        }
        cx.notify();
    }

    fn update_ticker(&mut self, retrying: bool, cx: &Context<Self>) {
        if !retrying {
            self.ticker = None;
            return;
        }
        if self.ticker.is_some() {
            return;
        }
        self.ticker = Some(
            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            }),
        );
    }

    fn watch_child(&mut self, child: SessionId, cx: &Context<Self>) {
        if !self.children.watch(&child) {
            return;
        }
        let engine = self.services.engine().clone();
        let key = child.clone();
        let job = async move { engine.messages(&key).await };
        let task = runtime::spawn(cx, self.services.runtime(), job, move |this, result, cx| {
            match result {
                Ok(messages) => {
                    this.children.load(&child, &messages);
                    cx.notify();
                },
                Err(error) => {
                    tracing::debug!(session = %child, error = %error, "subtask progress not loaded");
                },
            }
        });
        self.loads.retain(|task| !task.is_ready());
        self.loads.push(task);
    }

    fn on_changes(&mut self, changes: Vec<Change>, cx: &mut Context<Self>) {
        let dirty = changes
            .into_iter()
            .any(|change| self.children.apply(&change));
        if dirty {
            cx.notify();
        }
    }
}

impl Focusable for TranscriptView {
    fn focus_handle(&self, _cx: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for TranscriptView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let header = self.render_header(cx);
        let body = self.render_body(cx);
        let footer = self.render_footer(cx);
        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(header)
            .child(body)
            .child(footer);
        let root = div()
            .id("transcript")
            .key_context(LIST_CONTEXT)
            .track_focus(&self.focus)
            .flex()
            .flex_row()
            .size_full()
            .bg(palette.surface.app)
            .child(main);
        let root = Self::bind_keys(root, cx);
        if self.changes_open {
            root.child(self.render_changes(cx))
        } else {
            root
        }
    }
}
