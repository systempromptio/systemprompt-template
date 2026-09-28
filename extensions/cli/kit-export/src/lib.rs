//! Library surface for exporting one configured marketplace as a portable kit.
//!
//! The CLI owns argument parsing and output-directory policy. This crate
//! exposes only the export and strict round-trip proof so callers can test or
//! embed the exact same filesystem transformation.

mod export;
mod skill_md;
mod verify;

pub use export::{ExportReport, export_kit};
pub use verify::{Diff, round_trip};
