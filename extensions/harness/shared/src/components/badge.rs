//! Small status marks: a toned badge or pill, and a status dot that can pulse
//! while work is live.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    App, ElementId, IntoElement, ParentElement as _, Pixels, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::icons::{Icon, IconName};
use crate::motion;
use crate::theme::{Tone, radius, size, space, text, tokens};

/// A short label on a tinted background. `pill()` rounds it fully.
#[derive(IntoElement, Debug, Clone)]
pub struct Badge {
    label: SharedString,
    tone: Tone,
    icon: Option<IconName>,
    pill: bool,
    solid: bool,
}

impl Badge {
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            tone: Tone::Neutral,
            icon: None,
            pill: false,
            solid: false,
        }
    }

    #[must_use]
    pub const fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub const fn pill(mut self) -> Self {
        self.pill = true;
        self
    }

    #[must_use]
    pub const fn solid(mut self) -> Self {
        self.solid = true;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = tokens(cx).tone(self.tone);
        let (bg, fg, border) = if self.solid {
            (colors.solid, colors.on_solid, colors.solid)
        } else {
            (colors.bg, colors.fg, colors.border)
        };
        div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .gap(space::XXS)
            .h(px(18.))
            .px(if self.pill { space::SM } else { space::XS })
            .rounded(if self.pill { radius::FULL } else { radius::SM })
            .border_1()
            .border_color(border)
            .bg(bg)
            .text_color(fg)
            .text_size(text::CAPTION)
            .whitespace_nowrap()
            .children(
                self.icon
                    .map(|icon| Icon::new(icon).size(px(12.)).color(fg)),
            )
            .child(self.label)
    }
}

/// A small coloured dot for a status (connected, syncing, failed). `pulse`
/// animates it while work is ongoing.
#[derive(IntoElement, Debug, Clone)]
pub struct StatusDot {
    id: ElementId,
    tone: Tone,
    size: Pixels,
    pulse: bool,
}

impl StatusDot {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, tone: Tone) -> Self {
        Self {
            id: id.into(),
            tone,
            size: px(8.),
            pulse: false,
        }
    }

    #[must_use]
    pub const fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub const fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }
}

impl RenderOnce for StatusDot {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens(cx);
        let color = match self.tone {
            Tone::Neutral => palette.text.muted,
            other => palette.tone(other).fg,
        };
        let dot = div()
            .flex_shrink_0()
            .size(self.size)
            .rounded(radius::FULL)
            .bg(color);
        let frame = div()
            .flex()
            .items_center()
            .justify_center()
            .size(size::ICON_SM)
            .flex_shrink_0();
        if self.pulse {
            frame.child(motion::pulse(self.id, dot, cx))
        } else {
            frame.child(dot)
        }
    }
}
