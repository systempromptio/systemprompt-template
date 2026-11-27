//! The docks above the composer: the permission request and the questions a
//! session is waiting on, one at a time, oldest first, with a count when more
//! are waiting.
//!
//! Contract (fixed by the shell): `DocksView::new(session, services, window,
//! cx)` is built once per open session by the frame and renders an empty
//! element when nothing is pending. It answers through
//! `SessionState::reply_permission` and `SessionState::answer_question`. A new
//! request takes keyboard focus; Enter and Esc come through the list bindings
//! (Activate, Dismiss), A and R as plain keys, and all of them are
//! accepted only after a short delay, so keys typed elsewhere a moment before
//! cannot answer it.

mod keys;
mod permission;
pub mod permission_info;
mod question;
pub mod question_draft;

use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::component::input;
use gpui_kit::{
    AnyElement, AppContext as _, Context, ElementId, Entity, FocusHandle, Focusable as _,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Subscription, Window, div,
};
use systemprompt_engine::run::{PermissionAsk, PermissionId, QuestionAsk, Reply};
use systemprompt_harness_shared::Patch;
use systemprompt_harness_shared::components::{Activate, Dismiss, LIST_CONTEXT, TextField};
use systemprompt_harness_shared::motion::{self, Edge};
use systemprompt_harness_shared::theme::space;

use crate::app::Services;
use crate::docks::question_draft::QuestionDraft;
use crate::state::SessionState;
use crate::state::session_model::SessionModel;
use crate::strings::docks as text;

const ARM_DELAY: Duration = Duration::from_millis(600);

#[derive(Debug, Clone, PartialEq)]
enum Head {
    Permission(PermissionAsk),
    Question(QuestionAsk),
}

impl Head {
    const fn id(&self) -> &str {
        match self {
            Self::Permission(ask) => ask.id.as_str(),
            Self::Question(ask) => ask.id.as_str(),
        }
    }
}

fn head(model: &SessionModel) -> Option<(Head, usize)> {
    let waiting = (model.permissions.len() + model.questions.len()).saturating_sub(1);
    let first = model
        .permissions
        .first()
        .map(|ask| Head::Permission(ask.clone()))
        .or_else(|| {
            model
                .questions
                .first()
                .map(|ask| Head::Question(ask.clone()))
        })?;
    Some((first, waiting))
}

struct QuestionForm {
    draft: QuestionDraft,
    typed: Vec<Entity<TextField>>,
}

pub(crate) struct DocksView {
    session: Entity<SessionState>,
    _services: Services,
    focus: FocusHandle,
    feedback: Entity<TextField>,
    rejecting: bool,
    shown: Option<String>,
    form: Option<QuestionForm>,
    diff: Option<(PermissionId, Arc<Patch>)>,
    pending_focus: bool,
    armed_at: Instant,
    restore_focus: Option<FocusHandle>,
    _observation: Subscription,
}

impl std::fmt::Debug for DocksView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocksView")
            .field("shown", &self.shown)
            .finish_non_exhaustive()
    }
}

impl DocksView {
    pub(crate) fn new(
        session: Entity<SessionState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe_in(&session, window, |this, _, window, cx| {
            this.sync(window, cx);
        });
        let feedback =
            cx.new(|cx| TextField::new(window, cx).placeholder(text::FEEDBACK_PLACEHOLDER));
        let mut docks = Self {
            session,
            _services: services,
            focus: cx.focus_handle(),
            feedback,
            rejecting: false,
            shown: None,
            form: None,
            diff: None,
            pending_focus: false,
            armed_at: Instant::now(),
            restore_focus: None,
            _observation: observation,
        };
        docks.sync(window, cx);
        docks
    }

    fn head(&self, cx: &Context<Self>) -> Option<(Head, usize)> {
        head(self.session.read(cx).model())
    }

    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.head(cx).map(|(head, _)| head);
        let id = current.as_ref().map(|head| head.id().to_owned());
        if id == self.shown {
            cx.notify();
            return;
        }
        let had_focus = self.focus.contains_focused(window, cx);
        self.shown = id;
        self.rejecting = false;
        self.diff = None;
        self.feedback.update(cx, |field, cx| {
            field.set_text(SharedString::default(), window, cx);
        });
        self.form = match &current {
            Some(Head::Question(ask)) => Some(QuestionForm {
                draft: QuestionDraft::new(&ask.questions),
                typed: ask
                    .questions
                    .iter()
                    .map(|_| cx.new(|cx| TextField::new(window, cx).placeholder(text::OWN_ANSWER)))
                    .collect(),
            }),
            _ => None,
        };
        if current.is_some() {
            self.pending_focus = true;
        } else if had_focus && let Some(previous) = self.restore_focus.take() {
            previous.focus(window, cx);
        }
        cx.notify();
    }

    fn armed(&self) -> bool {
        self.armed_at.elapsed() >= ARM_DELAY
    }

    fn reply(&self, id: &PermissionId, reply: Reply, cx: &mut Context<Self>) {
        self.session
            .update(cx, |state, cx| state.reply_permission(id, reply, cx));
    }

    fn reject_with_feedback(&self, id: &PermissionId, cx: &mut Context<Self>) {
        let typed = self.feedback.read(cx).text(cx).trim().to_owned();
        let feedback = (!typed.is_empty()).then_some(typed);
        self.reply(id, Reply::Reject { feedback }, cx);
    }

    fn start_rejecting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rejecting = true;
        let handle = self.feedback.read(cx).focus_handle(cx);
        handle.focus(window, cx);
        cx.notify();
    }

    fn typing(&self, window: &Window, cx: &Context<Self>) -> bool {
        let fields = self
            .form
            .iter()
            .flat_map(|form| form.typed.iter())
            .chain(std::iter::once(&self.feedback));
        fields
            .map(|field| field.read(cx).focus_handle(cx))
            .any(|handle| handle.contains_focused(window, cx))
    }

    fn take_focus(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.pending_focus = false;
        self.armed_at = Instant::now();
        if !self.focus.contains_focused(window, cx) {
            self.restore_focus = window.focused(cx);
        }
        let handle = self.focus.clone();
        window.defer(cx, move |window, cx| handle.focus(window, cx));
    }
}

impl Render for DocksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some((head, waiting)) = self.head(cx) else {
            return div().w_full().into_any_element();
        };
        if self.pending_focus {
            self.take_focus(window, cx);
        }
        let content: AnyElement = match &head {
            Head::Permission(ask) => self.render_permission(ask, waiting, cx),
            Head::Question(ask) => self.render_question(ask, waiting, cx),
        };
        let animation = ElementId::Name(SharedString::from(format!("dock-{}", head.id())));
        div()
            .id("docks")
            .key_context(LIST_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &Activate, window, cx| this.primary(window, cx)))
            .on_action(cx.listener(|this, _: &Dismiss, window, cx| this.secondary(window, cx)))
            .on_action(cx.listener(|this, _: &input::Enter, _, cx| this.field_submitted(cx)))
            .on_action(
                cx.listener(|this, _: &input::Escape, window, cx| this.field_escaped(window, cx)),
            )
            .on_key_down(cx.listener(Self::on_key))
            .w_full()
            .px(space::LG)
            .pt(space::SM)
            .child(motion::slide_in(animation, Edge::Bottom, content, cx))
            .into_any_element()
    }
}
