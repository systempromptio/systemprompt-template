//! An sRGB colour that can be declared in a `const`, composited and measured
//! for contrast.
//!
//! Tokens are written as hex so palettes stay readable and testable without a
//! running app; they become the toolkit's `Hsla` only when painted.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{Fill, Hsla, Rgba};

pub const AA_TEXT: f64 = 4.5;
pub const AA_LARGE: f64 = 3.0;

/// An sRGB colour with straight (non-premultiplied) alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    rgb: u32,
    alpha: f32,
}

impl Color {
    #[must_use]
    pub const fn hex(rgb: u32) -> Self {
        Self {
            rgb: rgb & 0x00FF_FFFF,
            alpha: 1.0,
        }
    }

    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self {
            rgb: self.rgb,
            alpha,
        }
    }

    #[must_use]
    pub const fn rgb(self) -> u32 {
        self.rgb
    }

    #[must_use]
    pub const fn alpha(self) -> f32 {
        self.alpha
    }

    const fn channel_bytes(self) -> [u8; 3] {
        [
            ((self.rgb >> 16) & 0xFF) as u8,
            ((self.rgb >> 8) & 0xFF) as u8,
            (self.rgb & 0xFF) as u8,
        ]
    }

    fn from_bytes(bytes: [u8; 3], alpha: f32) -> Self {
        Self {
            rgb: (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]),
            alpha,
        }
    }

    #[must_use]
    pub fn relative_luminance(self) -> f64 {
        let [r, g, b] = self.channel_bytes().map(|byte| {
            let value = f64::from(byte) / 255.0;
            if value <= 0.040_45 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.0722f64.mul_add(b, 0.7152f64.mul_add(g, 0.2126 * r))
    }

    #[must_use]
    pub fn over(self, backdrop: Self) -> Self {
        let alpha = f64::from(self.alpha.clamp(0.0, 1.0));
        let top = self.channel_bytes();
        let bottom = backdrop.channel_bytes();
        let mut out = [0u8; 3];
        for (index, slot) in out.iter_mut().enumerate() {
            let mixed = f64::mul_add(
                f64::from(bottom[index]),
                1.0 - alpha,
                f64::from(top[index]) * alpha,
            );
            *slot = mixed.round().clamp(0.0, 255.0) as u8;
        }
        Self::from_bytes(out, backdrop.alpha)
    }

    #[must_use]
    pub fn mix(self, other: Self, amount: f32) -> Self {
        let blended = other
            .with_alpha(amount.clamp(0.0, 1.0))
            .over(self.with_alpha(1.0));
        blended.with_alpha(self.alpha)
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        let alpha = (self.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        if alpha == 255 {
            format!("#{:06x}", self.rgb)
        } else {
            format!("#{:06x}{alpha:02x}", self.rgb)
        }
    }

    #[must_use]
    pub fn hsla(self) -> Hsla {
        let [r, g, b] = self.channel_bytes().map(|byte| f32::from(byte) / 255.0);
        Hsla::from(Rgba {
            r,
            g,
            b,
            a: self.alpha.clamp(0.0, 1.0),
        })
    }
}

impl From<Color> for Hsla {
    fn from(color: Color) -> Self {
        color.hsla()
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Self::from(color.hsla())
    }
}

#[must_use]
pub fn contrast_ratio(foreground: Color, background: Color) -> f64 {
    let backdrop = background.with_alpha(1.0);
    let painted = if foreground.alpha() < 1.0 {
        foreground.over(backdrop)
    } else {
        foreground
    };
    let a = painted.relative_luminance();
    let b = backdrop.relative_luminance();
    let (light, dark) = if a >= b { (a, b) } else { (b, a) };
    (light + 0.05) / (dark + 0.05)
}
