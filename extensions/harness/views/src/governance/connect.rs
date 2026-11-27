//! The Add instance form: an address, an optional personal access token and
//! "keep me signed in", signing in through the browser or with the token. The
//! new instance joins the list; it becomes active only when none is.

use gpui_kit::{
    AppContext as _, Context, Entity, EventEmitter, Focusable as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Task, Window, div,
};
use harness_ui::Tone;
use harness_ui::components::{Banner, Button, Checkbox, Spinner, TextField};
use harness_ui::theme::{space, text as type_scale, tokens};
use sp_protocol::view::SignInOutcome;

use crate::app::{Services, runtime};
use crate::strings::governance as text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectEvent {
    SignedIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Browser,
    Token,
}

pub(crate) struct ConnectForm {
    services: Services,
    url: Entity<TextField>,
    token: Entity<TextField>,
    keep: bool,
    phase: Phase,
    message: Option<(Tone, &'static str)>,
    task: Option<Task<()>>,
}

impl std::fmt::Debug for ConnectForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectForm")
            .field("phase", &self.phase)
            .finish_non_exhaustive()
    }
}

impl EventEmitter<ConnectEvent> for ConnectForm {}

impl ConnectForm {
    pub(crate) fn new(
        services: Services,
        url: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let on_url = cx.entity().downgrade();
        let url = cx.new(|cx| {
            let field = TextField::new(window, cx)
                .label(text::CONNECT_URL_LABEL)
                .placeholder(text::CONNECT_URL_PLACEHOLDER)
                .on_submit(move |_, _, cx| {
                    if on_url.update(cx, Self::start_browser).is_err() {
                        tracing::trace!("connect form closed before submit");
                    }
                });
            match url {
                Some(url) => field.value(url),
                None => field,
            }
        });
        let on_token = cx.entity().downgrade();
        let token = cx.new(|cx| {
            TextField::new(window, cx)
                .label(text::CONNECT_TOKEN_LABEL)
                .hint(text::CONNECT_TOKEN_HINT)
                .password(true)
                .on_submit(move |_, _, cx| {
                    if on_token.update(cx, Self::start_token).is_err() {
                        tracing::trace!("connect form closed before submit");
                    }
                })
        });
        Self {
            services,
            url,
            token,
            keep: true,
            phase: Phase::Idle,
            message: None,
            task: None,
        }
    }

    pub(crate) fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.url.read(cx).focus_handle(cx);
        handle.focus(window, cx);
    }

    fn busy(&self) -> bool {
        self.phase != Phase::Idle
    }

    fn address(&self, cx: &mut Context<Self>) -> Option<String> {
        let url = self.url.read(cx).text(cx).trim().to_owned();
        let error = url
            .is_empty()
            .then_some(SharedString::new_static(text::CONNECT_URL_MISSING));
        self.url.update(cx, |field, cx| field.set_error(error, cx));
        (!url.is_empty()).then_some(url)
    }

    fn start_browser(&mut self, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        let Some(url) = self.address(cx) else { return };
        let sign_in = self.services.accounts().sign_in_browser(&url, self.keep);
        self.begin(Phase::Browser, cx);
        let task = runtime::spawn(cx, self.services.runtime(), sign_in.outcome(), Self::finish);
        self.task = Some(task);
    }

    fn start_token(&mut self, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        let Some(url) = self.address(cx) else { return };
        let token = self.token.read(cx).text(cx).trim().to_owned();
        if token.is_empty() {
            self.start_browser(cx);
            return;
        }
        let accounts = self.services.accounts().clone();
        let keep = self.keep;
        self.begin(Phase::Token, cx);
        let job = async move { accounts.sign_in_pat(&url, token, keep).await };
        let task = runtime::spawn(cx, self.services.runtime(), job, Self::finish);
        self.task = Some(task);
    }

    fn begin(&mut self, phase: Phase, cx: &mut Context<Self>) {
        self.phase = phase;
        self.message = None;
        cx.notify();
    }

    fn cancel(&self, cx: &mut Context<Self>) {
        self.services.accounts().cancel_sign_in();
        cx.notify();
    }

    fn finish(&mut self, outcome: SignInOutcome, cx: &mut Context<Self>) {
        self.phase = Phase::Idle;
        self.task = None;
        match outcome {
            SignInOutcome::SignedIn => cx.emit(ConnectEvent::SignedIn),
            SignInOutcome::Cancelled => {
                self.message = Some((Tone::Neutral, text::sign_in_outcome(outcome)));
            },
            other => self.message = Some((Tone::Danger, text::sign_in_outcome(other))),
        }
        cx.notify();
    }

    fn render_progress(&self, cx: &Context<Self>) -> Option<impl IntoElement> {
        let palette = *tokens(cx);
        let line = match self.phase {
            Phase::Idle => return None,
            Phase::Browser => text::CONNECT_WAITING,
            Phase::Token => text::CONNECT_WORKING,
        };
        let cancel = (self.phase == Phase::Browser).then(|| {
            Button::new("connect-cancel")
                .label(text::CANCEL)
                .small()
                .on_click(cx.listener(|this, _, _, cx| this.cancel(cx)))
        });
        Some(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::MD)
                .child(Spinner::new())
                .child(
                    div()
                        .flex_1()
                        .text_size(type_scale::SMALL)
                        .text_color(palette.text.secondary)
                        .child(line),
                )
                .children(cancel),
        )
    }
}

impl Render for ConnectForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let secure = self.services.accounts().snapshot().secure_storage;
        let busy = self.busy() || !secure;
        let keep_view = cx.entity().downgrade();
        let keep = Checkbox::new("connect-keep")
            .label(text::CONNECT_KEEP)
            .checked(self.keep)
            .disabled(self.busy())
            .on_change(move |checked, _, cx| {
                if keep_view
                    .update(cx, |form, cx| {
                        form.keep = checked;
                        cx.notify();
                    })
                    .is_err()
                {
                    tracing::trace!("connect form closed before the change");
                }
            });
        let message = self.message.map(|(tone, message)| {
            let color = palette.tone(tone).fg;
            div()
                .text_size(type_scale::SMALL)
                .text_color(color)
                .child(message)
        });
        div()
            .flex()
            .flex_col()
            .gap(space::LG)
            .children(
                (!secure).then(|| Banner::warning("connect-insecure", text::INSECURE_STORAGE)),
            )
            .child(self.url.clone())
            .child(self.token.clone())
            .child(keep)
            .children(self.render_progress(cx))
            .children(message)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(space::MD)
                    .child(
                        Button::new("connect-token")
                            .label(text::CONNECT_TOKEN)
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.start_token(cx))),
                    )
                    .child(
                        Button::new("connect-browser")
                            .label(text::CONNECT_BROWSER)
                            .primary()
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.start_browser(cx))),
                    ),
            )
    }
}
