//! Restrained motion: short fades and slides for surfaces that appear, and a
//! slow pulse for live work.
//!
//! Every helper honours the reduced-motion flag (`App::reduce_motion`, seeded
//! from the operating system by the toolkit, settable by the shell): with it
//! set, the element is returned in its final state with no animation attached.

use std::time::Duration;

use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, ElementId, IntoElement, ParentElement as _,
    Pixels, Styled as _, div, ease_out_quint, pulsating_between, px,
};

pub const FAST: Duration = Duration::from_millis(120);
pub const NORMAL: Duration = Duration::from_millis(180);
pub const PULSE: Duration = Duration::from_millis(1600);
pub const SLIDE_DISTANCE: Pixels = px(6.);

/// The edge a sliding element enters from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Edge {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

#[must_use]
pub fn reduced(cx: &App) -> bool {
    cx.reduce_motion()
}

pub fn fade_in(id: impl Into<ElementId>, child: impl IntoElement, cx: &App) -> AnyElement {
    let frame = div().child(child);
    if reduced(cx) {
        return frame.into_any_element();
    }
    frame
        .with_animation(
            id,
            Animation::new(NORMAL).with_easing(ease_out_quint()),
            gpui_kit::Styled::opacity,
        )
        .into_any_element()
}

pub fn slide_in(
    id: impl Into<ElementId>,
    edge: Edge,
    child: impl IntoElement,
    cx: &App,
) -> AnyElement {
    let frame = div().relative().child(child);
    if reduced(cx) {
        return frame.into_any_element();
    }
    let distance = SLIDE_DISTANCE.as_f32();
    frame
        .with_animation(
            id,
            Animation::new(NORMAL).with_easing(ease_out_quint()),
            move |this, t| {
                let travel = (1.0 - t) * distance;
                let this = this.opacity(t);
                match edge {
                    Edge::Top => this.top(px(-travel)),
                    Edge::Bottom => this.top(px(travel)),
                    Edge::Left => this.left(px(-travel)),
                    Edge::Right => this.left(px(travel)),
                }
            },
        )
        .into_any_element()
}

pub fn pulse(id: impl Into<ElementId>, child: impl IntoElement, cx: &App) -> AnyElement {
    let frame = div().child(child);
    if reduced(cx) {
        return frame.into_any_element();
    }
    frame
        .with_animation(
            id,
            Animation::new(PULSE)
                .repeat()
                .with_easing(pulsating_between(0.35, 1.0)),
            gpui_kit::Styled::opacity,
        )
        .into_any_element()
}
