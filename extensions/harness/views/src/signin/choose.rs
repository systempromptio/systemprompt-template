//! Step 2 of the sign-in gate and its ending: the signed-in instances with the
//! chosen one's counts and manifest key fingerprint (information only; the gate
//! never trusts a key), Finish, and the "Finishing setup" wait.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, ClickEvent, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Styled as _, div,
};
use harness_ui::components::{
    Activate, Badge, Button, Card, Dismiss, LIST_CONTEXT, ListRow, ProgressBar, SelectNext,
    SelectPrevious, Spinner,
};
use harness_ui::theme::{self, radius, space, tokens};
use harness_ui::{IconName, Tone};

use super::address::{InstanceChoice, grouped_fingerprint};
use super::connect::{failure_line, note};
use super::flow::{Direction, Event};
use super::{SignInGate, send};
use crate::strings::signin as copy;

impl SignInGate {
    pub(super) fn render_choose(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let back = Button::new("signin-back")
            .label(copy::BACK)
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.apply(Event::Back, cx)));
        if self.flow.instances.is_empty() {
            return div()
                .flex()
                .flex_col()
                .gap(space::LG)
                .child(note(copy::NO_INSTANCES, palette.text.secondary))
                .child(div().child(back))
                .into_any_element();
        }
        let finish = Button::new("signin-finish")
            .label(copy::FINISH)
            .primary()
            .disabled(self.flow.selected_instance().is_none())
            .on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| this.apply(Event::FinishRequested, cx)),
            );
        div()
            .flex()
            .flex_col()
            .gap(space::LG)
            .child(self.render_list(cx))
            .children(
                self.flow
                    .selected_instance()
                    .map(|choice| render_detail(choice, cx)),
            )
            .children(
                self.flow
                    .choose_error
                    .map(|failure| failure_line(copy::failure(failure), cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(back)
                    .child(div().flex_1())
                    .child(finish),
            )
            .into_any_element()
    }

    fn render_list(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let rows = self
            .flow
            .instances
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                let gate = cx.weak_entity();
                let account = choice.account.clone();
                let trailing = choice.summary.as_ref().map_or_else(
                    || Spinner::new().into_any_element(),
                    |summary| {
                        Badge::new(copy::skills(summary.skills))
                            .tone(Tone::Accent)
                            .into_any_element()
                    },
                );
                ListRow::new(("signin-instance", index), choice.label.clone())
                    .subtitle(copy::instance_detail(&choice.host, choice.user.as_deref()))
                    .icon(IconName::Cloud)
                    .tall()
                    .selected(self.flow.selected.as_ref() == Some(&choice.account))
                    .trailing(trailing)
                    .on_click(move |_, _, cx| send(&gate, Event::Select(account.clone()), cx))
            });
        div()
            .key_context(LIST_CONTEXT)
            .track_focus(&self.list_focus)
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| {
                this.apply(Event::Move(Direction::Next), cx);
            }))
            .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| {
                this.apply(Event::Move(Direction::Previous), cx);
            }))
            .on_action(
                cx.listener(|this, _: &Activate, _, cx| this.apply(Event::FinishRequested, cx)),
            )
            .on_action(cx.listener(|this, _: &Dismiss, _, cx| this.apply(Event::Back, cx)))
            .flex()
            .flex_col()
            .gap(space::XXS)
            .p(space::XS)
            .rounded(radius::LG)
            .border_1()
            .border_color(palette.border.subtle)
            .focus_visible(|style| style.border_color(palette.border.focus))
            .bg(palette.surface.panel)
            .children(rows)
            .into_any_element()
    }

    pub(super) fn render_finishing(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let label = self
            .flow
            .selected_instance()
            .map_or_else(String::new, |choice| choice.label.clone());
        let detail = self
            .flow
            .synced
            .map_or_else(|| copy::finishing(&label), copy::first_sync);
        div()
            .flex()
            .flex_col()
            .gap(space::MD)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(space::MD)
                    .child(Spinner::new().accent())
                    .child(note(detail, palette.text.secondary).flex_1()),
            )
            .child(ProgressBar::new("signin-finishing").label(copy::FINISHING_TITLE))
            .into_any_element()
    }
}

fn render_detail(choice: &InstanceChoice, cx: &Context<SignInGate>) -> AnyElement {
    let palette = *tokens(cx);
    let counts: SharedString = choice.summary.as_ref().map_or_else(
        || copy::COUNTS_LOADING.into(),
        |summary| copy::counts(summary).into(),
    );
    let role = choice
        .summary
        .as_ref()
        .and_then(|summary| summary.role.as_deref())
        .map(copy::role);
    let key = choice.fingerprint.as_ref().map_or_else(
        || div().child(note(copy::FINGERPRINT_NONE, palette.text.muted)),
        |fingerprint| {
            let (badge, tone) = if choice.trusted {
                (copy::KEY_TRUSTED, Tone::Success)
            } else {
                (copy::KEY_NOT_TRUSTED, Tone::Neutral)
            };
            div()
                .flex()
                .flex_col()
                .gap(space::XS)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(space::SM)
                        .child(note(copy::FINGERPRINT_LABEL, palette.text.secondary))
                        .child(Badge::new(badge).tone(tone).icon(IconName::Shield)),
                )
                .child(
                    div()
                        .font_family(theme::mono_font(cx))
                        .text_size(theme::text::MONO)
                        .text_color(palette.text.primary)
                        .child(grouped_fingerprint(fingerprint.as_str())),
                )
                .child(note(copy::FINGERPRINT_NOTE, palette.text.muted))
        },
    );
    Card::new()
        .title(choice.label.clone())
        .child(note(counts, palette.text.secondary))
        .children(role.map(|role| note(role, palette.text.secondary)))
        .child(key)
        .into_any_element()
}
