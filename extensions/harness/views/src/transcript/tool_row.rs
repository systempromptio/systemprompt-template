//! One compact row per tool call: icon, tool name, its salient argument, the
//! live title, duration and status, with denials and rejections called out,
//! expanding to the call's details.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_store::{Part, ToolState};
use harness_ui::components::{Badge, Spinner};
use harness_ui::theme::{mono_font, radius, size, space, text as type_scale, tokens};
use harness_ui::{Icon, IconName, Palette, Tone};

use crate::strings::transcript as text;
use crate::views::transcript::TranscriptView;
use crate::views::transcript::parts::part_element_id;
use crate::views::transcript::tool_info::{self, Outcome, Phase, ToolKind, ToolSummary};

const TITLE_WIDTH: f32 = 320.0;

impl TranscriptView {
    pub(super) fn render_tool(
        &mut self,
        part: &Part,
        tool: &str,
        state: &ToolState,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let summary = tool_info::summarize(tool, state);
        let open = self.expanded.contains(&part.id);
        let header = self.tool_header(part, &summary, state, cx);
        let refused = matches!(
            summary.phase,
            Phase::Failed(Outcome::PolicyDenied(_) | Outcome::RuleDenied(_))
        );
        let border = if refused {
            palette.danger.border
        } else {
            palette.border.subtle
        };
        let mut column = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(radius::MD)
            .border_1()
            .border_color(border)
            .bg(palette.surface.app)
            .child(header);
        if let Phase::Failed(outcome) = &summary.phase
            && let Some(note) = outcome_note(outcome, &palette)
        {
            column = column.child(note);
        }
        if open {
            let body = self.render_tool_body(part, &summary, state, cx);
            column = column.child(
                div()
                    .px(space::MD)
                    .pb(space::MD)
                    .pt(space::XS)
                    .border_t_1()
                    .border_color(palette.border.subtle)
                    .child(body),
            );
        }
        column.into_any_element()
    }

    fn tool_header(
        &self,
        part: &Part,
        summary: &ToolSummary,
        state: &ToolState,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let (id, message) = (part.id.clone(), part.message_id.clone());
        let open = self.expanded.contains(&part.id);
        let title = summary
            .title
            .clone()
            .filter(|title| summary.subject.as_ref() != Some(title));
        let chevron = if open {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        };
        div()
            .id(part_element_id("tool", &part.id))
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .w_full()
            .h(px(28.))
            .px(space::MD)
            .cursor_pointer()
            .text_size(type_scale::SMALL)
            .hover(|style| style.bg(palette.surface.hover))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_part(&id, &message, cx)))
            .child(
                Icon::new(summary.kind.icon())
                    .size(size::ICON_SM)
                    .color(palette.text.secondary),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(palette.text.primary)
                    .child(SharedString::from(summary.name.clone())),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_family(mono_font(cx))
                    .text_color(palette.text.secondary)
                    .child(SharedString::from(
                        summary.subject.clone().unwrap_or_default(),
                    )),
            )
            .children(title.map(|title| {
                div()
                    .max_w(px(TITLE_WIDTH))
                    .truncate()
                    .text_color(palette.text.muted)
                    .child(SharedString::from(title))
            }))
            .children(summary.duration_ms.map(|ms| {
                div()
                    .flex_shrink_0()
                    .text_color(palette.text.muted)
                    .child(SharedString::from(text::duration(ms)))
            }))
            .child(status_mark(summary, state, &palette))
            .child(
                Icon::new(chevron)
                    .size(size::ICON_SM)
                    .color(palette.text.muted),
            )
            .into_any_element()
    }
}

fn status_mark(summary: &ToolSummary, state: &ToolState, palette: &Palette) -> AnyElement {
    let icon = |name: IconName, color| {
        Icon::new(name)
            .size(size::ICON_SM)
            .color(color)
            .into_any_element()
    };
    match &summary.phase {
        Phase::Pending | Phase::Running => Spinner::new().size(size::ICON_SM).into_any_element(),
        Phase::Done
            if summary.kind == ToolKind::Shell
                && tool_info::exit_code(state).is_some_and(|code| code != 0) =>
        {
            icon(IconName::Alert, palette.warning.fg)
        },
        Phase::Done => icon(IconName::Check, palette.success.fg),
        Phase::Failed(Outcome::PolicyDenied(_)) => icon(IconName::Shield, palette.danger.fg),
        Phase::Failed(Outcome::RuleDenied(_)) => icon(IconName::Shield, palette.warning.fg),
        Phase::Failed(Outcome::Rejected(_)) => icon(IconName::CircleX, palette.warning.fg),
        Phase::Failed(Outcome::Aborted) => icon(IconName::Stop, palette.text.muted),
        Phase::Failed(Outcome::Error(_)) => icon(IconName::CircleX, palette.danger.fg),
    }
}

fn outcome_note(outcome: &Outcome, palette: &Palette) -> Option<AnyElement> {
    let (badge, tone, detail) = match outcome {
        Outcome::PolicyDenied(reason) => (text::POLICY_DENIED, Tone::Danger, Some(reason.clone())),
        Outcome::RuleDenied(reason) => (text::RULE_DENIED, Tone::Warning, Some(reason.clone())),
        Outcome::Rejected(feedback) => (text::REJECTED, Tone::Warning, feedback.clone()),
        Outcome::Error(error) => (
            text::FAILED,
            Tone::Danger,
            error.lines().next().map(str::to_owned),
        ),
        Outcome::Aborted => return None,
    };
    let detail = detail
        .filter(|detail| !detail.trim().is_empty())
        .unwrap_or_else(|| text::NO_REASON.to_owned());
    let colors = palette.tone(tone);
    Some(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .px(space::MD)
            .py(space::XS)
            .bg(colors.bg)
            .text_size(type_scale::SMALL)
            .child(Badge::new(badge).tone(tone).icon(IconName::Shield))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(palette.text.primary)
                    .child(SharedString::from(detail)),
            )
            .into_any_element(),
    )
}
