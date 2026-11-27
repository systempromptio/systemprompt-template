//! The built-in palette: a neutral graphite with a blue accent.

mod graphite;

pub(super) use graphite::{GRAPHITE_DARK, GRAPHITE_LIGHT};

use super::color::Color;

const fn c(red: u8, green: u8, blue: u8) -> Color {
    Color::hex(u32::from_be_bytes([0, red, green, blue]))
}
