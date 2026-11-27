//! The sync part of an instance card: how its sync stands, what it holds and
//! which signing key it trusts.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use super::page_view::LABEL_WIDTH;
use crate::strings::governance as text;
use crate::views::governance::model::{InstanceModel, KeyTrust};
use crate::views::governance::page::{GovernancePage, Target};
use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement as _, Styled as _, div, px,
};
use harness_ui::components::{Button, StatusDot};
use harness_ui::theme::{mono_font, radius, space, text as type_scale, tokens};
use harness_ui::{Icon, IconName, Tone};
use sp_protocol::view::SyncCounts;

pub(super) fn sync_status(
    row: &InstanceModel,
    index: usize,
    cx: &Context<GovernancePage>,
) -> AnyElement {
    let palette = *tokens(cx);
    let hint = row.sync.failure().map(text::sync_error_hint);
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .text_size(type_scale::SMALL)
                .child(
                    StatusDot::new(("instance-sync", index), row.sync.tone())
                        .pulse(row.sync.live()),
                )
                .child(
                    div()
                        .text_color(palette.text.secondary)
                        .child(text::sync_line(&row.sync)),
                ),
        )
        .children(hint.map(|hint| {
            div()
                .pl(space::XL)
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(hint)
        }))
        .into_any_element()
}

pub(super) fn holdings(counts: &SyncCounts, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    let warnings = counts.warnings.iter().map(|warning| {
        div()
            .flex()
            .flex_row()
            .items_start()
            .gap(space::SM)
            .child(Icon::new(IconName::Alert).small().color(palette.warning.fg))
            .child(div().flex_1().min_w_0().child(warning.clone()))
    });
    div()
        .flex()
        .flex_col()
        .gap(space::XS)
        .text_size(type_scale::SMALL)
        .child(
            div()
                .text_color(palette.text.muted)
                .child(text::HOLDS_TITLE),
        )
        .child(div().child(text::counts(
            counts.skills,
            counts.agents,
            counts.mcp,
            counts.rules,
        )))
        .children((!counts.warnings.is_empty()).then(|| {
            div()
                .pt(space::XS)
                .text_color(palette.text.muted)
                .child(text::WARNINGS_TITLE)
        }))
        .children(warnings)
        .into_any_element()
}

pub(super) fn key_trust(
    key: &KeyTrust,
    index: usize,
    target: &Target,
    cx: &Context<GovernancePage>,
) -> AnyElement {
    let palette = *tokens(cx);
    let tone = palette.tone(Tone::Warning);
    let mono = mono_font(cx);
    let fingerprint = |label: &'static str, value: String| {
        div()
            .flex()
            .flex_row()
            .gap(space::MD)
            .child(
                div()
                    .w(px(LABEL_WIDTH))
                    .flex_shrink_0()
                    .text_color(palette.text.muted)
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_family(mono.clone())
                    .text_size(type_scale::MONO)
                    .child(value),
            )
    };
    let (id, offered) = (target.id.clone(), key.offered.clone());
    let trust = Button::new(("instance-trust", index))
        .label(text::trust_label(key))
        .primary()
        .small()
        .icon(IconName::Key)
        .on_click(cx.listener(move |page, _, _, cx| page.trust(id.clone(), offered.clone(), cx)));
    div()
        .flex()
        .flex_col()
        .gap(space::SM)
        .p(space::MD)
        .rounded(radius::LG)
        .border_1()
        .border_color(tone.border)
        .bg(tone.bg)
        .text_size(type_scale::SMALL)
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(tone.fg)
                .child(text::key_badge(key)),
        )
        .child(
            div()
                .text_color(palette.text.primary)
                .child(text::key_body(key)),
        )
        .children(
            key.pinned
                .as_ref()
                .map(|pinned| fingerprint(text::KEY_PINNED, pinned.as_str().to_owned())),
        )
        .child(fingerprint(
            text::KEY_OFFERED,
            key.offered.as_str().to_owned(),
        ))
        .child(div().flex().flex_row().justify_end().child(trust))
        .into_any_element()
}
