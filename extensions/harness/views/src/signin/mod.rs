//! The sign-in gate, shown over everything below the titlebar while no instance
//! is signed in and active.
//!
//! The flow itself is the pure state machine in `flow`; this view renders it,
//! feeds it events from the controls and from the work it asks for (`effects`,
//! run through `crate::app::runtime`), and keeps the keyboard to itself while
//! shown: focus stays inside the gate and the shell's own shortcuts are
//! swallowed. It engages when the accounts snapshot says sign-in is needed and,
//! once a sign-in succeeded here, holds the window through step 2 until Finish
//! has faded it out, even though the snapshot already counts as signed in.

pub mod address;
mod choose;
mod connect;
mod effects;
pub mod flow;
mod page;

use gpui_kit::{
    Animation, AnimationExt as _, App, AppContext as _, Context, Entity, FocusHandle,
    Focusable as _, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, WeakEntity, Window, div, ease_out_quint,
};
use harness_ui::components::TextField;
use harness_ui::motion;
use harness_ui::theme::tokens;

use crate::app::Services;
use crate::keymap::{
    CommandPalette, FocusComposer, NewSession, OpenSettings, StopRun, ToggleSidebar,
};
use crate::state::AppState;
use crate::strings::signin as copy;
use effects::Work;
use flow::{Event, Flow, Step};

const GATE_CONTEXT: &str = "HarnessSignIn";

enum UiRequest {
    FocusStep,
    FocusToken,
    ClearToken,
    ResetFields(String),
    Lock(bool),
}

pub(crate) struct SignInGate {
    app: Entity<AppState>,
    services: Services,
    flow: Flow,
    engaged: bool,
    locked: bool,
    address: Entity<TextField>,
    token: Entity<TextField>,
    focus: FocusHandle,
    list_focus: FocusHandle,
    requests: Vec<UiRequest>,
    work: Work,
    _observation: Subscription,
}

impl std::fmt::Debug for SignInGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignInGate")
            .field("step", &self.flow.step)
            .field("engaged", &self.engaged)
            .finish_non_exhaustive()
    }
}

impl SignInGate {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let gate = cx.weak_entity();
        let (on_edit, on_submit) = (gate.clone(), gate.clone());
        let address = cx.new(|cx| {
            TextField::new(window, cx)
                .label(copy::ADDRESS_LABEL)
                .placeholder(copy::ADDRESS_PLACEHOLDER)
                .on_change(move |text, _, cx| {
                    send_later(&on_edit, Event::Edited(text.to_string()), cx);
                })
                .on_submit(move |_, _, cx| send_later(&on_submit, Event::SettleNow, cx))
        });
        let (on_token_edit, on_token_submit) = (gate.clone(), gate);
        let token = cx.new(|cx| {
            TextField::new(window, cx)
                .label(copy::TOKEN_LABEL)
                .placeholder(copy::TOKEN_PLACEHOLDER)
                .password(true)
                .on_change(move |_, _, cx| send_later(&on_token_edit, Event::TokenEdited, cx))
                .on_submit(move |value, _, cx| {
                    send_later(
                        &on_token_submit,
                        Event::TokenSubmitted(address::credential_kind(value)),
                        cx,
                    );
                })
        });
        let observation = cx.observe(&app, |this, _, cx| this.follow_accounts(cx));
        let secure = app.read(cx).accounts.secure_storage;
        let mut gate = Self {
            app,
            services,
            flow: Flow::new("", secure),
            engaged: false,
            locked: false,
            address,
            token,
            focus: cx.focus_handle(),
            list_focus: cx.focus_handle(),
            requests: Vec::new(),
            work: Work::default(),
            _observation: observation,
        };
        gate.follow_accounts(cx);
        gate
    }

    pub(crate) const fn holds_window(&self) -> bool {
        self.engaged
    }

    fn follow_accounts(&mut self, cx: &mut Context<Self>) {
        let default = self.services.identity().default_instance();
        let (gated, secure, opening) = {
            let snapshot = &self.app.read(cx).accounts;
            (
                snapshot.needs_sign_in(),
                snapshot.secure_storage,
                address::opening_address(snapshot, default),
            )
        };
        if gated && !self.engaged {
            self.engage(&opening, secure, cx);
            return;
        }
        if secure != self.flow.secure {
            self.apply(Event::SecureStorage(secure), cx);
        }
        if !gated && self.engaged && self.flow.releasable() {
            self.engaged = false;
            cx.notify();
        }
    }

    fn engage(&mut self, opening: &str, secure: bool, cx: &mut Context<Self>) {
        tracing::debug!("sign-in gate engaged");
        self.engaged = true;
        self.work = Work::default();
        self.flow = Flow::new(opening, secure);
        self.requests
            .push(UiRequest::ResetFields(opening.to_owned()));
        self.requests.push(UiRequest::FocusStep);
        let effects = self.flow.start();
        self.run_all(effects, cx);
        cx.notify();
    }

    fn submit_token(&mut self, cx: &mut Context<Self>) {
        let kind = address::credential_kind(&self.token.read(cx).text(cx));
        self.apply(Event::TokenSubmitted(kind), cx);
    }

    fn escape(&mut self, cx: &mut Context<Self>) {
        let event = if self.flow.step == Step::Choose {
            Event::Back
        } else {
            Event::CancelRequested
        };
        self.apply(event, cx);
    }

    fn flush_requests(&mut self, window: &Window, cx: &mut Context<Self>) {
        let requests = std::mem::take(&mut self.requests);
        let refocus = self.engaged && !self.focus.contains_focused(window, cx);
        if requests.is_empty() && !refocus {
            return;
        }
        cx.defer_in(window, move |this, window, cx| {
            let mut focus_step = refocus;
            for request in requests {
                match request {
                    UiRequest::FocusStep => focus_step = true,
                    other => this.perform(other, window, cx),
                }
            }
            if focus_step {
                this.focus_step(window, cx);
            }
        });
    }

    fn perform(&self, request: UiRequest, window: &mut Window, cx: &mut Context<Self>) {
        match request {
            UiRequest::FocusStep => self.focus_step(window, cx),
            UiRequest::FocusToken => {
                let handle = self.token.read(cx).focus_handle(cx);
                handle.focus(window, cx);
            },
            UiRequest::ClearToken => self
                .token
                .update(cx, |field, cx| field.set_text("", window, cx)),
            UiRequest::ResetFields(opening) => {
                self.address
                    .update(cx, |field, cx| field.set_text(opening, window, cx));
                self.token
                    .update(cx, |field, cx| field.set_text("", window, cx));
            },
            UiRequest::Lock(locked) => {
                self.address
                    .update(cx, |field, cx| field.set_disabled(locked, cx));
                self.token
                    .update(cx, |field, cx| field.set_disabled(locked, cx));
            },
        }
    }

    fn focus_step(&self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = match self.flow.step {
            Step::Connect if self.flow.token_only() => self.token.read(cx).focus_handle(cx),
            Step::Connect => self.address.read(cx).focus_handle(cx),
            Step::Choose if !self.flow.instances.is_empty() => self.list_focus.clone(),
            _ => self.focus.clone(),
        };
        handle.focus(window, cx);
    }
}

pub(super) fn send(gate: &WeakEntity<SignInGate>, event: Event, cx: &mut App) {
    if gate.update(cx, |gate, cx| gate.apply(event, cx)).is_err() {
        tracing::trace!("sign-in event arrived after the gate closed");
    }
}

fn send_later(gate: &WeakEntity<SignInGate>, event: Event, cx: &mut App) {
    let gate = gate.clone();
    cx.defer(move |cx| send(&gate, event, cx));
}

impl Render for SignInGate {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.flush_requests(window, cx);
        let palette = *tokens(cx);
        let page = self.render_page(cx);
        let content = if self.flow.step == Step::Closing && !motion::reduced(cx) {
            div()
                .size_full()
                .child(page)
                .with_animation(
                    "signin-fade-out",
                    Animation::new(motion::NORMAL).with_easing(ease_out_quint()),
                    |this, delta| this.opacity(1.0 - delta),
                )
                .into_any_element()
        } else {
            page
        };
        div()
            .id("signin-gate")
            .key_context(GATE_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &NewSession, _, _| {}))
            .on_action(cx.listener(|_, _: &FocusComposer, _, _| {}))
            .on_action(cx.listener(|_, _: &ToggleSidebar, _, _| {}))
            .on_action(cx.listener(|_, _: &OpenSettings, _, _| {}))
            .on_action(cx.listener(|_, _: &CommandPalette, _, _| {}))
            .on_action(cx.listener(|this, _: &StopRun, _, cx| this.escape(cx)))
            .size_full()
            .bg(palette.surface.app)
            .text_color(palette.text.primary)
            .child(content)
    }
}
