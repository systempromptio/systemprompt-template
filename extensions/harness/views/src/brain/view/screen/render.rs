//! The Brain screen drawn: why it cannot be used yet, or the tab bar over the
//! facts tab or the proposals inbox.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Window, div,
};
use harness_ui::IconName;
use harness_ui::components::{Button, EmptyState, IconButton, Spinner};
use harness_ui::theme::{space, tokens};

use super::{BrainScreen, Tab};
use crate::model::{Availability, BrainFailure};
use crate::strings;

impl Render for BrainScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::mem::take(&mut self.clear_reason) {
            self.reason_field.update(cx, |field, cx| {
                field.set_text("", window, cx);
                field.set_error(None, cx);
            });
        }
        let availability = self.availability();
        if !availability.is_ready() {
            return div()
                .size_full()
                .child(unavailable(availability))
                .into_any_element();
        }
        let body = match self.tab {
            Tab::Facts => self.render_facts_tab(cx),
            Tab::Proposals => self.render_proposals(cx),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.render_tabs(cx))
            .child(div().flex_1().min_h_0().child(body))
            .into_any_element()
    }
}

impl BrainScreen {
    fn render_tabs(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let proposals: SharedString = match self.inbox.open_count() {
            0 => SharedString::new_static(strings::TAB_PROPOSALS),
            open => strings::open_proposals(open).into(),
        };
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .px(space::LG)
            .py(space::SM)
            .border_b_1()
            .border_color(palette.border.subtle)
            .child(
                Button::new("brain-tab-facts")
                    .label(strings::TAB_FACTS)
                    .small()
                    .ghost()
                    .selected(self.tab == Tab::Facts)
                    .on_click(cx.listener(|this, _, _, cx| this.set_tab(Tab::Facts, cx))),
            )
            .child(
                Button::new("brain-tab-proposals")
                    .label(proposals)
                    .small()
                    .ghost()
                    .selected(self.tab == Tab::Proposals)
                    .on_click(cx.listener(|this, _, _, cx| this.set_tab(Tab::Proposals, cx))),
            )
            .child(div().flex_1())
            .child(
                IconButton::new("brain-refresh", IconName::Sync, strings::REFRESH)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.reload_all(cx))),
            )
            .into_any_element()
    }

    pub(super) fn failed(failure: BrainFailure, cx: &Context<Self>) -> AnyElement {
        EmptyState::new(strings::failure(failure))
            .icon(IconName::Alert)
            .action(
                Button::new("brain-retry")
                    .label(strings::REFRESH)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.reload_all(cx))),
            )
            .into_any_element()
    }

    pub(super) fn waiting() -> AnyElement {
        div().p(space::LG).child(Spinner::new()).into_any_element()
    }
}

fn unavailable(availability: Availability) -> AnyElement {
    let icon = match availability {
        Availability::Withheld => IconName::Shield,
        Availability::Detached => IconName::Cloud,
        Availability::Checking => IconName::Sync,
        Availability::Disabled | Availability::Ready => IconName::Info,
    };
    let state = EmptyState::new(strings::availability_title(availability))
        .icon(icon)
        .description(strings::availability_body(availability));
    match availability {
        Availability::Checking => state.action(Spinner::new()).into_any_element(),
        _ => state.into_any_element(),
    }
}
