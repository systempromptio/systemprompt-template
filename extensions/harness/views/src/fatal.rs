//! The window shown when the app cannot start: what happened, where the details
//! are, and a way out.

use gpui_kit::{App, Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::{Button, EmptyState};
use systemprompt_harness_shared::theme::tokens;

use crate::strings::foundation as text;

#[derive(Debug, Clone, Copy, Default)]
pub struct FatalView;

impl Render for FatalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        div().size_full().bg(palette.surface.app).child(
            EmptyState::new(text::STARTUP_FAILED_TITLE)
                .icon(IconName::Alert)
                .description(text::STARTUP_FAILED_BODY)
                .action(
                    Button::new("fatal-quit")
                        .label(text::QUIT)
                        .primary()
                        .on_click(|_, _, cx: &mut App| cx.quit()),
                ),
        )
    }
}
