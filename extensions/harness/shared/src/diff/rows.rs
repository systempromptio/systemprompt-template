//! The rows of a diff view: file headers, hunk headers and numbered, tinted
//! lines.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{Div, ParentElement as _, Pixels, SharedString, Styled as _, div, px};

use super::model::{DiffLine, FilePatch, FileStatus, Hunk, LineKind};
use super::stat::DiffStat;
use crate::icons::{Icon, IconName};
use crate::theme::{Palette, size, space, text};

const MONO_CHAR: f32 = 7.5;

pub(super) struct RowStyle<'a> {
    pub(super) palette: &'a Palette,
    pub(super) mono: SharedString,
    pub(super) gutter: Pixels,
}

impl<'a> RowStyle<'a> {
    pub(super) fn for_hunk(hunk: &Hunk, palette: &'a Palette, mono: SharedString) -> Self {
        let widest = (hunk.old_start + hunk.old_len).max(hunk.new_start + hunk.new_len);
        let digits = widest.max(1).to_string().len() as f32;
        Self {
            palette,
            mono,
            gutter: px(digits.mul_add(MONO_CHAR, 12.)),
        }
    }
}

pub(super) fn line_row(line: &DiffLine, style: &RowStyle<'_>) -> Div {
    let p = style.palette;
    let (bg, sign, sign_color) = match line.kind {
        LineKind::Added => (Some(p.diff.added_bg), "+", p.diff.added_fg),
        LineKind::Removed => (Some(p.diff.removed_bg), "-", p.diff.removed_fg),
        LineKind::Context => (None, " ", p.text.muted),
        LineKind::NoNewline => (None, "\\", p.text.muted),
    };
    let number = |value: Option<u32>| {
        div()
            .flex_shrink_0()
            .w(style.gutter)
            .pr(space::SM)
            .text_right()
            .text_color(p.text.muted)
            .child(
                value
                    .map(|n| SharedString::from(n.to_string()))
                    .unwrap_or_default(),
            )
    };
    let marker = line.kind == LineKind::NoNewline;
    let content = div()
        .flex_shrink_0()
        .pr(space::LG)
        .text_color(if marker { p.text.muted } else { p.text.primary })
        .child(SharedString::from(line.text.clone()));
    let content = if marker { content.italic() } else { content };
    let mut row = div()
        .flex()
        .flex_row()
        .w_full()
        .whitespace_nowrap()
        .font_family(style.mono.clone())
        .text_size(text::MONO)
        .line_height(text::MONO_LINE)
        .child(number(line.old_line))
        .child(number(line.new_line))
        .child(
            div()
                .flex_shrink_0()
                .w(px(14.))
                .text_color(sign_color)
                .child(SharedString::new_static(sign)),
        )
        .child(content);
    if let Some(bg) = bg {
        row = row.bg(bg);
    }
    row
}

pub(super) fn file_header(file: &FilePatch, palette: &Palette) -> Div {
    let path = match (file.status, &file.old_path, &file.new_path) {
        (FileStatus::Renamed, Some(old), Some(new)) => format!("{old} \u{2192} {new}"),
        _ => file.display_path().to_owned(),
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::SM)
        .w_full()
        .px(space::MD)
        .py(space::XS)
        .bg(palette.surface.panel)
        .border_b_1()
        .border_color(palette.border.subtle)
        .text_size(text::SMALL)
        .child(
            Icon::new(IconName::File)
                .size(size::ICON_SM)
                .color(palette.text.muted),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(palette.text.primary)
                .child(SharedString::from(path)),
        )
        .child(DiffStat::from(file.stat()))
}

pub(super) fn hunk_header(hunk: &Hunk, style: &RowStyle<'_>) -> Div {
    let mut header = format!(
        "@@ -{},{} +{},{} @@",
        hunk.old_start, hunk.old_len, hunk.new_start, hunk.new_len
    );
    if !hunk.section.is_empty() {
        header.push(' ');
        header.push_str(&hunk.section);
    }
    div()
        .w_full()
        .px(space::MD)
        .bg(style.palette.diff.hunk_bg)
        .text_color(style.palette.text.muted)
        .font_family(style.mono.clone())
        .text_size(text::MONO)
        .line_height(text::MONO_LINE)
        .whitespace_nowrap()
        .truncate()
        .child(SharedString::from(header))
}
