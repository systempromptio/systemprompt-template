//! The expanded details of a tool call: the shell command and its output, the
//! diff an edit, write or patch made, the todo checklist, a subagent's
//! progress, or the raw input and output of any other tool.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div, px,
};
use harness_store::{Part, TodoStatus, ToolState};
use harness_ui::components::{Badge, Button};
use harness_ui::theme::{size, space, text as type_scale, tokens};
use harness_ui::{CodeBlock, DiffView, Icon, IconName, Tone};
use serde_json::Value;

use crate::strings::transcript as text;
use crate::views::transcript::parts::part_element_id;
use crate::views::transcript::tool_info::{self, Outcome, Phase, TodoLine, ToolKind, ToolSummary};
use crate::views::transcript::{TranscriptEvent, TranscriptView};

const OUTPUT_HEIGHT: f32 = 320.0;
const DIFF_HEIGHT: f32 = 420.0;

impl TranscriptView {
    pub(super) fn render_tool_body(
        &mut self,
        part: &Part,
        summary: &ToolSummary,
        state: &ToolState,
        cx: &Context<Self>,
    ) -> AnyElement {
        if let Phase::Failed(Outcome::Error(error)) = &summary.phase {
            return output_block(part, text::OUTPUT, error);
        }
        let diff = tool_info::metadata_text(state, "diff").map(str::to_owned);
        match summary.kind {
            ToolKind::Shell => shell_body(part, state),
            kind if kind.writes_files() && diff.is_some() => {
                let source = diff.unwrap_or_default();
                let patch = self.patches.patch(&part.id, &source);
                DiffView::new(part_element_id("tool-diff", &part.id), patch)
                    .max_height(px(DIFF_HEIGHT))
                    .fold_label(|hidden| SharedString::from(text::folded(hidden)))
                    .into_any_element()
            },
            ToolKind::Todo => checklist(&tool_info::todos(state), cx),
            ToolKind::Task => self.task_body(part, state, cx),
            _ => generic_body(part, state),
        }
    }

    fn task_body(&mut self, part: &Part, state: &ToolState, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let child = tool_info::child_session(state);
        let progress = child
            .as_ref()
            .and_then(|child| self.children.progress(child));
        let line = progress.map_or_else(
            || text::SUBTASK_STARTING.to_owned(),
            |progress| text::subtask_progress(progress.steps, progress.current.as_deref()),
        );
        let mut top = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .text_size(type_scale::SMALL)
            .text_color(palette.text.secondary)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(SharedString::from(line)),
            );
        if let Some(child) = child {
            top = top.child(
                Button::new(part_element_id("task-open", &part.id))
                    .label(text::OPEN_SUBTASK)
                    .icon(IconName::ExternalLink)
                    .small()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(TranscriptEvent::OpenSession(child.clone()));
                    })),
            );
        }
        let mut column = div().flex().flex_col().gap(space::SM).child(top);
        if let ToolState::Completed { output, .. } = state
            && !output.trim().is_empty()
        {
            column = column.child(self.render_text(&part.id, output, false, false));
        }
        column.into_any_element()
    }
}

fn shell_body(part: &Part, state: &ToolState) -> AnyElement {
    let command = state
        .input()
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let shell = tool_info::metadata_text(state, "shell")
        .unwrap_or("shell")
        .to_owned();
    let mut column = div().flex().flex_col().gap(space::SM).child(
        CodeBlock::new(part_element_id("shell-command", &part.id), command)
            .label(shell)
            .copy_button(text::COPY),
    );
    let mut badges = div().flex().flex_row().gap(space::XS);
    let exit_code = tool_info::exit_code(state).filter(|code| *code != 0);
    let timed_out = tool_info::metadata(state)
        .and_then(|map| map.get("timed_out"))
        .and_then(Value::as_bool)
        == Some(true);
    if let Some(code) = exit_code {
        badges =
            badges.child(Badge::new(format!("{} {code}", text::EXIT_CODE)).tone(Tone::Warning));
    }
    if timed_out {
        badges = badges.child(Badge::new(text::TIMED_OUT).tone(Tone::Warning));
    }
    if exit_code.is_some() || timed_out {
        column = column.child(badges);
    }
    let output = tool_info::output(state)
        .unwrap_or(text::NO_OUTPUT)
        .to_owned();
    column
        .child(output_block(part, text::OUTPUT, &output))
        .into_any_element()
}

fn output_block(part: &Part, label: &'static str, output: &str) -> AnyElement {
    CodeBlock::new(part_element_id("tool-output", &part.id), output.to_owned())
        .label(label)
        .max_height(px(OUTPUT_HEIGHT))
        .copy_button(text::COPY)
        .into_any_element()
}

fn generic_body(part: &Part, state: &ToolState) -> AnyElement {
    // Why: discard-ok: pretty printing a JSON value cannot fail; the compact form
    // stands in if it ever does.
    let input =
        serde_json::to_string_pretty(state.input()).unwrap_or_else(|_| state.input().to_string());
    let mut column = div().flex().flex_col().gap(space::SM).child(
        CodeBlock::new(part_element_id("tool-input", &part.id), input)
            .language("json")
            .label(text::INPUT)
            .max_height(px(OUTPUT_HEIGHT)),
    );
    if let Some(output) = tool_info::output(state) {
        column = column.child(output_block(part, text::OUTPUT, output));
    }
    column.into_any_element()
}

fn checklist(lines: &[TodoLine], cx: &Context<TranscriptView>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .text_size(type_scale::SMALL)
        .children(lines.iter().map(|line| {
            let (icon, color) = match line.status {
                TodoStatus::Completed => (IconName::CircleCheck, palette.success.fg),
                TodoStatus::InProgress => (IconName::ChevronRight, palette.text.accent),
                TodoStatus::Cancelled => (IconName::CircleX, palette.text.muted),
                TodoStatus::Pending => (IconName::Minus, palette.text.muted),
            };
            let label = div()
                .flex_1()
                .min_w_0()
                .child(SharedString::from(line.content.clone()));
            let label = match line.status {
                TodoStatus::Completed | TodoStatus::Cancelled => {
                    label.line_through().text_color(palette.text.muted)
                },
                TodoStatus::InProgress => label.text_color(palette.text.primary),
                TodoStatus::Pending => label.text_color(palette.text.secondary),
            };
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .child(Icon::new(icon).size(size::ICON_SM).color(color))
                .child(label)
                .child(
                    div()
                        .text_color(palette.text.muted)
                        .child(text::todo_marker(line.status)),
                )
        }))
        .into_any_element()
}
