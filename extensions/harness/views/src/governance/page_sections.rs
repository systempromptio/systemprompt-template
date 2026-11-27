//! The sections of the Governance page below the instances: its header with Add
//! instance, the content synced from the active instance, the verdict switch
//! and the heartbeat line.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _,
    div,
};
use harness_ui::components::{Badge, Banner, Button, ListRow, SectionHeader, StatusDot, Toggle};
use harness_ui::theme::{space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};

use crate::strings::governance as text;
use crate::views::governance::feeds::{Held, HeldServer, McpState};
use crate::views::governance::heartbeat::HeartbeatLine;
use crate::views::governance::page::GovernancePage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerdictState {
    pub(crate) enabled: bool,
    pub(crate) available: bool,
    pub(crate) governed: Option<bool>,
}

pub(crate) fn header(title: &'static str, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(space::LG)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(space::XS)
                .child(
                    div()
                        .text_size(type_scale::HEADING)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(type_scale::BODY)
                        .text_color(palette.text.secondary)
                        .child(text::PAGE_SUBTITLE),
                ),
        )
        .child(
            Button::new("governance-add-instance")
                .label(text::ADD_INSTANCE)
                .icon(IconName::Plus)
                .primary()
                .on_click(cx.listener(|page, _, window, cx| page.open_connect(None, window, cx))),
        )
        .into_any_element()
}

pub(crate) fn insecure_banner() -> Banner {
    Banner::warning("governance-insecure", text::INSECURE_STORAGE)
}

fn section(title: impl Into<SharedString>, body: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(space::XS)
        .child(SectionHeader::new(title))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(space::XS)
                .px(space::MD)
                .child(body),
        )
        .into_any_element()
}

fn muted(message: impl Into<SharedString>, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .text_size(type_scale::SMALL)
        .text_color(palette.text.muted)
        .child(message.into())
        .into_any_element()
}

pub(crate) fn synced(name: &str, held: Option<&Held>, cx: &Context<GovernancePage>) -> AnyElement {
    let Some(held) = held else {
        return section(text::synced_from(name), muted(text::SYNCED_EMPTY, cx));
    };
    let skills: Vec<AnyElement> = held
        .skills
        .iter()
        .enumerate()
        .map(|(index, skill)| {
            let row = ListRow::new(
                ("synced-skill", index),
                SharedString::from(skill.name.clone()),
            )
            .icon(IconName::Sparkle);
            match &skill.description {
                Some(description) => row.subtitle(SharedString::from(description.clone())).tall(),
                None => row,
            }
            .into_any_element()
        })
        .collect();
    let servers: Vec<AnyElement> = held
        .servers
        .iter()
        .enumerate()
        .map(|(index, server)| server_row(index, server).into_any_element())
        .collect();
    let body = div()
        .flex()
        .flex_col()
        .gap(space::SM)
        .child(group(text::SKILLS, skills, cx))
        .child(group(text::MCP_SERVERS, servers, cx))
        .child(group(
            text::RULES,
            vec![muted(text::rules_line(held.rules), cx)],
            cx,
        ));
    section(text::synced_from(name), body)
}

fn group(title: &'static str, rows: Vec<AnyElement>, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    let count = rows.len();
    let rows = if rows.is_empty() {
        vec![muted(text::NONE_SYNCED, cx)]
    } else {
        rows
    };
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .child(
            div()
                .flex()
                .flex_row()
                .gap(space::SM)
                .text_size(type_scale::SMALL)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text.secondary)
                .child(title)
                .children((title != text::RULES).then(|| {
                    div()
                        .text_color(palette.text.muted)
                        .child(count.to_string())
                })),
        )
        .children(rows)
        .into_any_element()
}

fn server_row(index: usize, server: &HeldServer) -> ListRow {
    let (label, tone): (SharedString, Tone) = match server.state {
        McpState::Connected { tools } => (text::mcp_connected(tools).into(), Tone::Success),
        McpState::Connecting => (text::MCP_CONNECTING.into(), Tone::Accent),
        McpState::Failed => (text::MCP_FAILED.into(), Tone::Danger),
        McpState::Disabled => (text::MCP_DISABLED.into(), Tone::Neutral),
        McpState::NeedsAuth => (text::MCP_NEEDS_AUTH.into(), Tone::Warning),
        McpState::NotStarted => (text::MCP_NOT_STARTED.into(), Tone::Neutral),
    };
    ListRow::new(
        ("synced-server", index),
        SharedString::from(server.name.clone()),
    )
    .icon(IconName::Plug)
    .trailing(Badge::new(label).tone(tone))
}

pub(crate) fn verdict(state: VerdictState, cx: &Context<GovernancePage>) -> AnyElement {
    let page = cx.entity().downgrade();
    let toggle = Toggle::new("governance-verdict")
        .label(text::VERDICT_LABEL)
        .on(state.enabled)
        .disabled(!state.available)
        .on_change(move |enabled, _, cx| {
            if page
                .update(cx, |page, cx| page.set_verdict(enabled, cx))
                .is_err()
            {
                tracing::trace!("governance page closed before the verdict change");
            }
        });
    let note = match (state.available, state.governed) {
        (false, _) => Some(text::VERDICT_UNAVAILABLE),
        (true, Some(false)) => Some(text::VERDICT_NOT_GOVERNED),
        (true, _) => None,
    };
    let body = div()
        .flex()
        .flex_col()
        .gap(space::XS)
        .child(toggle)
        .child(muted(text::VERDICT_BODY, cx))
        .children(note.map(|note| muted(note, cx)));
    section(text::VERDICT_TITLE, body)
}

pub(crate) fn heartbeat(line: &HeartbeatLine, cx: &Context<GovernancePage>) -> AnyElement {
    let palette = *tokens(cx);
    let body = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::SM)
        .text_size(type_scale::SMALL)
        .child(StatusDot::new("governance-heartbeat", line.tone()))
        .child(
            div()
                .text_color(palette.text.secondary)
                .child(text::heartbeat(line)),
        );
    section(text::HEARTBEAT_TITLE, body)
}
