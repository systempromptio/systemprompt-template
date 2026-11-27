//! An inline `+added −removed` line count.

use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div,
};

use super::model::Stat;
use crate::theme::{self, space, text};

/// An inline `+added −removed` count.
#[derive(IntoElement, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffStat {
    stat: Stat,
}

impl DiffStat {
    #[must_use]
    pub const fn new(added: usize, removed: usize) -> Self {
        Self {
            stat: Stat { added, removed },
        }
    }
}

impl From<Stat> for DiffStat {
    fn from(stat: Stat) -> Self {
        Self { stat }
    }
}

impl RenderOnce for DiffStat {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::tokens(cx);
        let added = palette.success.fg;
        let removed = palette.danger.fg;
        div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .gap(space::XS)
            .text_size(text::SMALL)
            .child(
                div()
                    .text_color(added)
                    .child(SharedString::from(format!("+{}", self.stat.added))),
            )
            .child(
                div()
                    .text_color(removed)
                    .child(SharedString::from(format!("\u{2212}{}", self.stat.removed))),
            )
    }
}
