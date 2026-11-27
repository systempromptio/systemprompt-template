//! The pill's menu: the instances to switch to, the fix for what needs
//! attention, Sync now and the settings page.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use super::pill::{GovernancePill, MENU_WIDTH};
use crate::strings::governance as text;
use crate::views::governance::model::Fix;
use crate::views::governance::pill_model::{PillModel, PillStatus};
use gpui_kit::{
    AnyElement, Context, Div, FontWeight, IntoElement, ParentElement as _, SharedString,
    Styled as _, div, px,
};
use harness_ui::IconName;
use harness_ui::components::{Button, Divider, ListRow, SectionHeader};
use harness_ui::theme::{space, text as type_scale, tokens};

impl GovernancePill {
    pub(super) fn render_menu(model: &PillModel, cx: &Context<Self>) -> AnyElement {
        let fix = model.status.fix().map(|fix| {
            Button::new("governance-pill-fix")
                .label(text::fix_label(fix))
                .primary()
                .small()
                .full_width()
                .on_click(cx.listener(move |this, _, _, cx| this.apply_fix(fix, cx)))
        });
        let sync_row = (model.can_sync
            && !matches!(model.status.fix(), Some(Fix::SyncNow | Fix::Wait)))
        .then(|| {
            ListRow::new("governance-pill-sync", text::SYNC_NOW)
                .icon(IconName::Sync)
                .on_click(cx.listener(|this, _, _, cx| this.sync_now(cx)))
        });
        let switches: Vec<ListRow> = model
            .others
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let (id, name) = (target.id.clone(), target.name.clone());
                ListRow::new(
                    ("governance-pill-switch", index),
                    SharedString::from(target.name.clone()),
                )
                .icon(IconName::User)
                .on_click(
                    cx.listener(move |this, _, _, cx| this.switch_to(id.clone(), name.clone(), cx)),
                )
            })
            .collect();
        let has_switches = !switches.is_empty();
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w(px(MENU_WIDTH))
            .child(menu_heading(model, cx))
            .children(fix.map(|button| div().px(space::MD).child(button)))
            .children(sync_row)
            .children(has_switches.then(Divider::horizontal))
            .children(has_switches.then(|| SectionHeader::new(text::PILL_SWITCH_TO)))
            .children(switches)
            .child(Divider::horizontal())
            .child(
                ListRow::new("governance-pill-settings", text::PILL_OPEN_SETTINGS)
                    .icon(IconName::Settings)
                    .on_click(cx.listener(|this, _, _, cx| this.open_page(cx))),
            )
            .into_any_element()
    }
}

fn menu_heading(model: &PillModel, cx: &Context<GovernancePill>) -> Div {
    let palette = *tokens(cx);
    let hint = match model.status {
        PillStatus::Failed(kind) => Some(text::sync_error_hint(kind)),
        _ => None,
    };
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .px(space::MD)
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .truncate()
                .child(model.label.clone()),
        )
        .child(
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.secondary)
                .child(text::pill_status(&model.status)),
        )
        .children(hint.map(|hint| {
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(hint)
        }))
}
