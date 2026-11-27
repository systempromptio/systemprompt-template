//! Spacing, radius and type scales, and the font stacks the theme resolves
//! against installed fonts.
//!
//! The scales are tuned for a dense desktop tool: a 4 px grid, small radii, 13
//! px body text.

/// The 4 px spacing grid.
pub mod space {
    use gpui_kit::{Pixels, px};

    pub const NONE: Pixels = px(0.);
    pub const XXS: Pixels = px(2.);
    pub const XS: Pixels = px(4.);
    pub const SM: Pixels = px(6.);
    pub const MD: Pixels = px(8.);
    pub const LG: Pixels = px(12.);
    pub const XL: Pixels = px(16.);
    pub const XXL: Pixels = px(24.);
    pub const XXXL: Pixels = px(32.);
}

/// Corner radii.
pub mod radius {
    use gpui_kit::{Pixels, px};

    pub const NONE: Pixels = px(0.);
    pub const SM: Pixels = px(3.);
    pub const MD: Pixels = px(4.);
    pub const LG: Pixels = px(6.);
    pub const XL: Pixels = px(8.);
    pub const FULL: Pixels = px(9999.);
}

/// Font sizes and line heights.
pub mod text {
    use gpui_kit::{Pixels, px};

    pub const CAPTION: Pixels = px(11.);
    pub const SMALL: Pixels = px(12.);
    pub const BODY: Pixels = px(13.);
    pub const BODY_LARGE: Pixels = px(14.);
    pub const TITLE: Pixels = px(16.);
    pub const HEADING: Pixels = px(20.);
    pub const DISPLAY: Pixels = px(26.);
    pub const MONO: Pixels = px(12.);

    pub const LINE_TIGHT: f32 = 1.25;
    pub const LINE_NORMAL: f32 = 1.45;
    pub const MONO_LINE: Pixels = px(18.);
}

/// Control heights, so buttons, fields and rows line up.
pub mod size {
    use gpui_kit::{Pixels, px};

    pub const CONTROL_SM: Pixels = px(24.);
    pub const CONTROL_MD: Pixels = px(28.);
    pub const CONTROL_LG: Pixels = px(32.);
    pub const ROW: Pixels = px(28.);
    pub const ROW_TALL: Pixels = px(40.);
    pub const ICON_SM: Pixels = px(14.);
    pub const ICON_MD: Pixels = px(16.);
    pub const ICON_LG: Pixels = px(20.);
}

/// Font families in order of preference; the first installed one is used.
pub mod font {
    pub const UI_STACK: &[&str] = &[
        "Segoe UI Variable Text",
        "Segoe UI Variable",
        "Segoe UI",
        ".SystemUIFont",
    ];
    pub const MONO_STACK: &[&str] = &["Cascadia Code", "Cascadia Mono", "Consolas", "Menlo"];

    #[must_use]
    pub fn pick<'a>(stack: &[&'a str], installed: &[String]) -> Option<&'a str> {
        stack.iter().copied().find(|family| {
            *family == ".SystemUIFont" || installed.iter().any(|name| name == family)
        })
    }
}
