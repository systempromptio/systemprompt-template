//! The built-in palettes: a neutral graphite with a blue accent, and the
//! Astound brand.

mod astound;
mod graphite;

pub(super) use astound::{ASTOUND_DARK, ASTOUND_LIGHT};
pub(super) use graphite::{GRAPHITE_DARK, GRAPHITE_LIGHT};

use super::color::Color;

const fn c(red: u8, green: u8, blue: u8) -> Color {
    Color::hex(u32::from_be_bytes([0, red, green, blue]))
}
