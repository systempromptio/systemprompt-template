//! The upgrade gate: the previous release's schema, restored from
//! `tests/fixtures/schema/release-baseline.sql`, must install under the
//! current binary's installer and end up shaped exactly like a fresh install.
//!
//! Every other tier installs the declarative baseline on an empty database.
//! No deployed instance is empty: it carries the previous release's tables and
//! runs only the pending migrations. Those are two different code paths
//! through core's installer, and on 2026-09-14 the second one crash-looped
//! production while the first was green everywhere. This crate runs the
//! second path on every push.

pub mod catalog;

#[cfg(test)]
mod upgrade;
