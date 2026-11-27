//! The frame drawn: titlebar, sidebar, the routed main area and the toasts,
//! with the shell's keyboard commands and the requests extension views
//! dispatch to it.

use super::Frame;
use crate::keymap::{
    CommandPalette, FRAME_CONTEXT, FocusComposer, NewSession, OpenSettings, StopRun, ToggleSidebar,
};
use crate::state::{AppState, Route};
use crate::strings::foundation as text;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div,
};
use systemprompt_harness_shared::Tone;
use systemprompt_harness_shared::contract::DraftInSession;
use systemprompt_harness_shared::theme::tokens;

impl Render for Frame {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let state = self.app.read(cx);
        let gated = state.needs_sign_in() || self.signin.read(cx).holds_window();
        let sidebar_open = state.sidebar_open;
        let body = if gated {
            div().flex_1().min_h_0().child(self.signin.clone())
        } else {
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_h_0()
                .children(sidebar_open.then(|| self.sidebar.clone()))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .child(self.render_main(cx)),
                )
        };
        div()
            .id("frame")
            .key_context(FRAME_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &NewSession, _, cx| {
                this.app.update(cx, AppState::new_session);
            }))
            .on_action(
                cx.listener(|this, _: &FocusComposer, window, cx| this.focus_composer(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.app.update(cx, AppState::toggle_sidebar);
            }))
            .on_action(cx.listener(|this, _: &OpenSettings, _, cx| {
                this.app.update(cx, |app, cx| {
                    app.navigate(Route::Settings { page: None }, cx);
                });
            }))
            .on_action(cx.listener(|this, _: &StopRun, _, cx| this.stop_run(cx)))
            .on_action(cx.listener(|this, action: &DraftInSession, window, cx| {
                this.app
                    .update(cx, |app, cx| app.navigate(Route::Session, cx));
                this.draft(action.text.to_string(), window, cx);
            }))
            .on_action(cx.listener(|this, _: &CommandPalette, _, cx| {
                this.app.update(cx, |_, cx| {
                    AppState::notify_user(Tone::Info, text::COMMAND_PALETTE_UNAVAILABLE, cx);
                });
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(palette.surface.app)
            .text_color(palette.text.primary)
            .child(self.titlebar.clone())
            .child(body)
    }
}
