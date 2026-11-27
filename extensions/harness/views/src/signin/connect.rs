//! Step 1 of the sign-in gate: the instance address with its live check, the
//! browser sign-in with its waiting state, "keep me signed in", and the
//! personal access token or one-time code field.

use gpui_kit::{
    AnyElement, ClickEvent, Context, IntoElement, ParentElement as _, SharedString, Styled as _,
    div, px,
};
use systemprompt_harness_shared::components::{
    Banner, Button, Checkbox, Divider, KeyHint, Spinner, StatusDot,
};
use systemprompt_harness_shared::theme::{self, size, space, tokens};
use systemprompt_harness_shared::{Icon, IconName, Tone};

use super::flow::{Check, Event, Method};
use super::{SignInGate, send};
use crate::strings::signin as copy;

impl SignInGate {
    pub(super) fn render_connect(&self, cx: &Context<Self>) -> AnyElement {
        let gate = cx.weak_entity();
        let keep = Checkbox::new("signin-keep")
            .label(copy::KEEP)
            .checked(self.flow.keep)
            .disabled(self.flow.pending.is_some())
            .on_change(move |keep, _, cx| send(&gate, Event::Keep(keep), cx));
        div()
            .flex()
            .flex_col()
            .gap(space::LG)
            .children((!self.flow.secure).then(|| {
                Banner::error("signin-insecure", copy::INSECURE_BODY).title(copy::INSECURE_TITLE)
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XS)
                    .child(self.address.clone())
                    .child(self.render_check(cx)),
            )
            .child(self.render_browser(cx))
            .child(keep)
            .child(Divider::horizontal())
            .child(self.render_token(cx))
            .into_any_element()
    }

    fn render_check(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let (tone, message): (Option<Tone>, SharedString) = match &self.flow.check {
            Check::Empty => (None, copy::ADDRESS_EMPTY.into()),
            Check::Typing => (None, SharedString::default()),
            Check::Checking(_) => (Some(Tone::Info), copy::ADDRESS_CHECKING.into()),
            Check::Ready(target) => (
                Some(Tone::Success),
                copy::found(&target.label, &target.host).into(),
            ),
            Check::Invalid(problem) => (Some(Tone::Danger), copy::problem(*problem).into()),
        };
        let color = if tone == Some(Tone::Danger) {
            palette.danger.fg
        } else {
            palette.text.muted
        };
        let checking = matches!(self.flow.check, Check::Checking(_));
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .min_h(px(18.))
            .text_size(theme::text::SMALL)
            .text_color(color)
            .children(tone.map(|tone| StatusDot::new("signin-check", tone).pulse(checking)))
            .child(message)
            .into_any_element()
    }

    fn render_browser(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let column = div().flex().flex_col().gap(space::SM);
        if self.flow.token_only() {
            return column
                .child(note(copy::TOKEN_ONLY, palette.text.secondary))
                .into_any_element();
        }
        let waiting = self.flow.pending == Some(Method::Browser);
        let button = Button::new("signin-browser")
            .label(copy::BROWSER)
            .icon(IconName::Globe)
            .primary()
            .large()
            .full_width()
            .loading(waiting)
            .disabled(!self.flow.can_sign_in())
            .on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| this.apply(Event::BrowserRequested, cx)),
            );
        let status = if waiting {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::MD)
                .child(Spinner::new().size(size::ICON_SM))
                .child(note(copy::BROWSER_WAITING, palette.text.secondary).flex_1())
                .child(KeyHint::new([copy::ESCAPE_KEY]))
                .child(
                    Button::new("signin-cancel")
                        .label(copy::CANCEL)
                        .small()
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.apply(Event::CancelRequested, cx);
                        })),
                )
                .into_any_element()
        } else {
            note(copy::BROWSER_HINT, palette.text.muted).into_any_element()
        };
        column
            .child(button)
            .child(status)
            .children(self.render_error(Method::Browser, cx))
            .into_any_element()
    }

    fn render_token(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let token_only = self.flow.token_only();
        let mut column = div().flex().flex_col().gap(space::SM);
        if !token_only {
            let label = if self.flow.token_open {
                copy::TOKEN_HIDE
            } else {
                copy::TOKEN_SHOW
            };
            column = column.child(
                div().child(
                    Button::new("signin-token-toggle")
                        .label(label)
                        .icon(IconName::Key)
                        .ghost()
                        .small()
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.apply(Event::ToggleToken, cx);
                        })),
                ),
            );
        }
        if !self.flow.token_open && !token_only {
            return column.into_any_element();
        }
        let connecting = self.flow.pending == Some(Method::Token);
        let ready = matches!(self.flow.check, Check::Ready(_));
        let mut connect = Button::new("signin-token-connect")
            .label(copy::TOKEN_CONNECT)
            .loading(connecting)
            .disabled(!self.flow.can_sign_in())
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.submit_token(cx)));
        if token_only {
            connect = connect.primary();
        }
        let actions = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .child(connect)
            .child(
                Button::new("signin-code-page")
                    .label(copy::CODE_PAGE)
                    .icon(IconName::ExternalLink)
                    .ghost()
                    .small()
                    .disabled(!ready || self.flow.pending.is_some())
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.apply(Event::CodePageRequested, cx);
                    })),
            );
        column
            .child(self.token.clone())
            .child(actions)
            .children(
                self.flow
                    .code_page_opened
                    .then(|| note(copy::CODE_PAGE_OPENED, palette.text.secondary)),
            )
            .children(self.render_error(Method::Token, cx))
            .into_any_element()
    }

    pub(super) fn render_error(&self, method: Method, cx: &Context<Self>) -> Option<AnyElement> {
        let (failed, failure) = self.flow.error?;
        (failed == method).then(|| failure_line(copy::failure(failure), cx))
    }
}

pub(super) fn failure_line(message: &'static str, cx: &Context<SignInGate>) -> AnyElement {
    let color = tokens(cx).danger.fg;
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(space::SM)
        .text_size(theme::text::SMALL)
        .text_color(color)
        .child(
            div()
                .pt(space::XXS)
                .child(Icon::new(IconName::Alert).size(size::ICON_SM).color(color)),
        )
        .child(div().flex_1().child(message))
        .into_any_element()
}

pub(super) fn note(message: impl Into<SharedString>, color: theme::Color) -> gpui_kit::Div {
    div()
        .text_size(theme::text::SMALL)
        .text_color(color)
        .child(message.into())
}
