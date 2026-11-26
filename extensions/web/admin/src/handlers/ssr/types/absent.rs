//! The view-model slot of a capability this build does not compile.
//!
//! A page section that needs an optional member keeps its field in every
//! flavour: with the member it holds the section's view, without it
//! `Absent`, which serialises to `null`. The strict-mode template branches on
//! the field (`{{#if usage}}`) rather than reading a key that is not there,
//! and no flavour renders a zero that reads as "no usage".

use serde::Serialize;

/// The value of a view-model field whose member is not linked into this build.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct Absent;
