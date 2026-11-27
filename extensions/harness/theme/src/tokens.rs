//! The semantic token set every component paints with.

use super::color::Color;

/// Background layers, from the window base up to floating surfaces, plus
/// interaction states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surfaces {
    pub app: Color,
    pub panel: Color,
    pub raised: Color,
    pub sunken: Color,
    pub hover: Color,
    pub active: Color,
    pub selected: Color,
    pub overlay: Color,
}

/// Foreground text levels. Every level but `disabled` meets AA on every
/// surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextColors {
    pub primary: Color,
    pub secondary: Color,
    pub muted: Color,
    pub disabled: Color,
    pub on_accent: Color,
    pub accent: Color,
}

/// Lines: hairlines between areas, control outlines, separators and the
/// keyboard focus ring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Borders {
    pub subtle: Color,
    pub strong: Color,
    pub divider: Color,
    pub focus: Color,
}

/// The brand's interactive colour in its resting, hovered and pressed fills,
/// and a quiet tint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accent {
    pub base: Color,
    pub hover: Color,
    pub active: Color,
    pub subtle: Color,
}

/// One feedback tone: text on surfaces, a tinted background, its border, and a
/// solid fill with its text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
    pub solid: Color,
    pub on_solid: Color,
}

/// Unified diff tints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffColors {
    pub added_bg: Color,
    pub removed_bg: Color,
    pub added_fg: Color,
    pub removed_fg: Color,
    pub hunk_bg: Color,
}

/// Syntax colours handed to the toolkit's highlighter; each meets AA on code
/// backgrounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxPalette {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub function: Color,
    pub type_name: Color,
    pub number: Color,
    pub constant: Color,
    pub operator: Color,
    pub punctuation: Color,
    pub variable: Color,
    pub property: Color,
    pub tag: Color,
    pub attribute: Color,
}

impl SyntaxPalette {
    #[must_use]
    pub const fn named(&self) -> [(&'static str, Color); 13] {
        [
            ("keyword", self.keyword),
            ("string", self.string),
            ("comment", self.comment),
            ("function", self.function),
            ("type", self.type_name),
            ("number", self.number),
            ("constant", self.constant),
            ("operator", self.operator),
            ("punctuation", self.punctuation),
            ("variable", self.variable),
            ("property", self.property),
            ("tag", self.tag),
            ("attribute", self.attribute),
        ]
    }
}

/// The feedback tone of a badge, banner, dot or toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
}

/// A complete set of tokens for one appearance (light or dark) of one brand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub surface: Surfaces,
    pub text: TextColors,
    pub border: Borders,
    pub accent: Accent,
    pub success: StatusColors,
    pub warning: StatusColors,
    pub danger: StatusColors,
    pub info: StatusColors,
    pub diff: DiffColors,
    pub syntax: SyntaxPalette,
    pub selection: Color,
    pub scrollbar: Color,
}

impl Palette {
    #[must_use]
    pub const fn tone(&self, tone: Tone) -> StatusColors {
        match tone {
            Tone::Neutral => StatusColors {
                fg: self.text.secondary,
                bg: self.surface.hover,
                border: self.border.subtle,
                solid: self.text.secondary,
                on_solid: self.surface.raised,
            },
            Tone::Accent => StatusColors {
                fg: self.text.accent,
                bg: self.accent.subtle,
                border: self.accent.subtle,
                solid: self.accent.base,
                on_solid: self.text.on_accent,
            },
            Tone::Success => self.success,
            Tone::Warning => self.warning,
            Tone::Danger => self.danger,
            Tone::Info => self.info,
        }
    }

    #[must_use]
    pub const fn text_surfaces(&self) -> [(&'static str, Color); 6] {
        [
            ("app", self.surface.app),
            ("panel", self.surface.panel),
            ("raised", self.surface.raised),
            ("sunken", self.surface.sunken),
            ("hover", self.surface.hover),
            ("selected", self.surface.selected),
        ]
    }
}
