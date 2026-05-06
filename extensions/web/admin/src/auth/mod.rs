//! Admin auth helpers — role-based capability gates.
//!
//! Today the project does not have a generic `Principal` extractor; instead
//! every SSR handler uses an `Extension<UserContext>`. This module centralizes
//! the capability checks performed against that context so that, when a richer
//! `Principal` arrives, only this file changes.

pub mod capabilities;

pub use capabilities::can_view_raw_transcript;
