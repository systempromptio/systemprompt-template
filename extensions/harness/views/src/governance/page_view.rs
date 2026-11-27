//! The instance cards of the Governance page: name and badges, address, user,
//! organisation and key, how the sync stands with its fix, what the active
//! instance holds, the key waiting for trust, and the per-instance actions.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use super::page_sync::{holdings, key_trust, sync_status};
use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _,
    div, px,
};
use harness_ui::components::{Avatar, Badge, Button, ButtonKind, EmptyState};
use harness_ui::theme::{mono_font, radius, space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};

use crate::strings::governance as text;
use crate::views::governance::model::{InstanceAction, InstanceModel};
use crate::views::governance::page::{GovernancePage, Target};

pub(super) const LABEL_WIDTH: f32 = 112.;

pub(crate) fn instance_list(rows: &[InstanceModel], cx: &Context<GovernancePage>) -> AnyElement {
    if rows.is_empty() {
        return div()
            .h(px(240.))
            .child(
                EmptyState::new(text::NO_INSTANCES_TITLE)
                    .icon(IconName::Shield)
                    .description(text::NO_INSTANCES_BODY),
            )
            .into_any_element();
    }
    div()
        .flex()
        .flex_col()
        .gap(space::LG)
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, row)| instance_card(row, index, cx)),
        )
        .into_any_element()
}

fn instance_card(row: &InstanceModel, index: usize, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    let target = Target {
        id: row.id.clone(),
        name: row.name.clone(),
        url: row.url.clone(),
    };
    div()
        .flex()
        .flex_col()
        .gap(space::MD)
        .p(space::LG)
        .rounded(radius::XL)
        .border_1()
        .border_color(if row.active {
            palette.border.strong
        } else {
            palette.border.subtle
        })
        .bg(palette.surface.raised)
        .child(header(row, index, &target, cx))
        .child(details(row, cx))
        .child(sync_status(row, index, cx))
        .children(row.holds.as_ref().map(|counts| holdings(counts, cx)))
        .children(
            row.key
                .as_ref()
                .map(|key| key_trust(key, index, &target, cx)),
        )
        .into_any_element()
}

fn header(
    row: &InstanceModel,
    index: usize,
    target: &Target,
    cx: &Context<GovernancePage>,
) -> AnyElement {
    let mut badges: Vec<Badge> = Vec::new();
    if row.active && row.signed_in {
        badges.push(Badge::new(text::BADGE_ACTIVE).tone(Tone::Success).pill());
    }
    if !row.signed_in {
        badges.push(
            Badge::new(text::BADGE_SIGNED_OUT)
                .tone(Tone::Warning)
                .pill(),
        );
    }
    if let Some(key) = &row.key {
        badges.push(
            Badge::new(text::key_badge(key))
                .tone(Tone::Warning)
                .icon(IconName::Key)
                .pill(),
        );
    }
    let buttons: Vec<Button> = row
        .actions
        .iter()
        .filter_map(|action| action_button(*action, index, target, cx))
        .collect();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::MD)
        .child(Avatar::new(SharedString::from(row.name.clone())).size(px(28.)))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_w_0()
                .items_center()
                .gap(space::SM)
                .child(
                    div()
                        .truncate()
                        .text_size(type_scale::BODY_LARGE)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(row.name.clone()),
                )
                .children(badges),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_shrink_0()
                .gap(space::SM)
                .children(buttons),
        )
        .into_any_element()
}

fn action_button(
    action: InstanceAction,
    index: usize,
    target: &Target,
    cx: &Context<GovernancePage>,
) -> Option<Button> {
    let (id, label, kind) = match action {
        InstanceAction::MakeActive => (
            "instance-make-active",
            text::MAKE_ACTIVE,
            ButtonKind::Primary,
        ),
        InstanceAction::SyncNow => ("instance-sync-now", text::SYNC_NOW, ButtonKind::Secondary),
        InstanceAction::SignIn => ("instance-sign-in", text::SIGN_IN_AGAIN, ButtonKind::Primary),
        InstanceAction::SignOut => ("instance-sign-out", text::SIGN_OUT, ButtonKind::Ghost),
        InstanceAction::Remove => ("instance-remove", text::REMOVE, ButtonKind::Ghost),
        InstanceAction::Trust => return None,
    };
    let target = target.clone();
    Some(
        Button::new((id, index))
            .label(label)
            .kind(kind)
            .small()
            .on_click(
                cx.listener(move |page, _, window, cx| page.run(action, &target, window, cx)),
            ),
    )
}

fn details(row: &InstanceModel, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    let mono = mono_font(cx);
    let line = |label: &'static str, value: String, monospace: bool| {
        let value = div()
            .flex_1()
            .min_w_0()
            .truncate()
            .text_color(palette.text.primary)
            .child(value);
        let value = if monospace {
            value.font_family(mono.clone()).text_size(type_scale::MONO)
        } else {
            value
        };
        div()
            .flex()
            .flex_row()
            .gap(space::MD)
            .text_size(type_scale::SMALL)
            .child(
                div()
                    .w(px(LABEL_WIDTH))
                    .flex_shrink_0()
                    .text_color(palette.text.muted)
                    .child(label),
            )
            .child(value)
    };
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .child(line(text::FIELD_URL, row.url.clone(), false))
        .child(line(
            text::FIELD_USER,
            row.user
                .clone()
                .unwrap_or_else(|| text::NOT_KNOWN.to_owned()),
            false,
        ))
        .child(line(
            text::FIELD_ORGANISATION,
            row.organisation
                .clone()
                .unwrap_or_else(|| text::NOT_KNOWN.to_owned()),
            false,
        ))
        .child(line(
            text::FIELD_KEY,
            row.fingerprint
                .clone()
                .unwrap_or_else(|| text::NO_KEY.to_owned()),
            true,
        ))
        .into_any_element()
}
