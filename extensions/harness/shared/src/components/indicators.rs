//! Progress and identity marks: spinner, progress bar, key hints and initials
//! avatars.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::hash::{Hash as _, Hasher as _};

use gpui_kit::component::Sizable as _;
use gpui_kit::component::progress::Progress;
use gpui_kit::component::spinner::Spinner as ToolkitSpinner;
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement as _, Pixels, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::theme::{Tone, radius, size, space, text, tokens};

/// An indeterminate spinner in the muted text colour. Static under reduced
/// motion.
#[derive(IntoElement, Debug, Clone, Copy)]
pub struct Spinner {
    size: Pixels,
    accent: bool,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            size: size::ICON_MD,
            accent: false,
        }
    }

    #[must_use]
    pub const fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub const fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = tokens(cx);
        let color = if self.accent {
            p.text.accent
        } else {
            p.text.muted
        };
        ToolkitSpinner::new()
            .with_size(self.size)
            .color(color.hsla())
    }
}

/// A thin progress bar: a fraction from 0 to 1, or indeterminate.
#[derive(IntoElement, Debug, Clone)]
pub struct ProgressBar {
    id: ElementId,
    fraction: Option<f32>,
    label: Option<SharedString>,
}

impl ProgressBar {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            fraction: None,
            label: None,
        }
    }

    #[must_use]
    pub const fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = Some(fraction.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = tokens(cx).accent.base;
        let mut bar = Progress::new(self.id).small().color(color.hsla());
        bar = match self.fraction {
            Some(fraction) => bar.value(fraction * 100.0),
            None => bar.loading(true),
        };
        if let Some(label) = self.label {
            bar = bar.accessibility_label(label);
        }
        bar
    }
}

/// Keycaps for a shortcut, e.g. `["Ctrl", "Enter"]`. Callers pass the key names
/// in the user's language.
#[derive(IntoElement, Debug, Clone)]
pub struct KeyHint {
    keys: Vec<SharedString>,
}

impl KeyHint {
    #[must_use]
    pub fn new<S: Into<SharedString>>(keys: impl IntoIterator<Item = S>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl RenderOnce for KeyHint {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = *tokens(cx);
        div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .gap(space::XXS)
            .children(self.keys.into_iter().map(|key| {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_w(px(18.))
                    .h(px(18.))
                    .px(space::XS)
                    .rounded(radius::SM)
                    .border_1()
                    .border_color(p.border.subtle)
                    .bg(p.surface.raised)
                    .text_size(text::CAPTION)
                    .text_color(p.text.secondary)
                    .child(key)
            }))
    }
}

#[must_use]
pub fn initials(name: &str) -> String {
    let local = name.split('@').next().unwrap_or(name);
    let words: Vec<&str> = local
        .split(|c: char| c.is_whitespace() || c == '.' || c == '_' || c == '-' || c == '@')
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .collect();
    let first_letter = |word: &str| word.chars().find(|c| c.is_alphanumeric());
    let letters: Vec<char> = match words.as_slice() {
        [] => Vec::new(),
        [only] => only
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(2)
            .collect(),
        [first, .., last] => first_letter(first)
            .into_iter()
            .chain(first_letter(last))
            .collect(),
    };
    letters.into_iter().flat_map(char::to_uppercase).collect()
}

const AVATAR_TONES: [Tone; 5] = [
    Tone::Accent,
    Tone::Info,
    Tone::Success,
    Tone::Warning,
    Tone::Danger,
];

fn avatar_tone(name: &str) -> Tone {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    AVATAR_TONES[(hasher.finish() % AVATAR_TONES.len() as u64) as usize]
}

/// A round avatar with the initials of a name, tinted by a colour derived from
/// the name.
#[derive(IntoElement, Debug, Clone)]
pub struct Avatar {
    name: SharedString,
    size: Pixels,
}

impl Avatar {
    #[must_use]
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            size: px(24.),
        }
    }

    #[must_use]
    pub const fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = tokens(cx).tone(avatar_tone(&self.name));
        let font_size = px((self.size.as_f32() * 0.42).max(9.0));
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(self.size)
            .rounded(radius::FULL)
            .bg(colors.bg)
            .text_color(colors.fg)
            .text_size(font_size)
            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
            .child(SharedString::from(initials(&self.name)))
    }
}
