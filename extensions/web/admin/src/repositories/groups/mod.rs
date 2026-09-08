//! Persistence for groups: the rows themselves, their membership, and the AD
//! groups that map onto them.
//!
//! Split by concern rather than by statement kind because membership is the
//! only part with two writers — the directory replaces its own rows at every
//! sign-in while an admin's manual rows survive — and that rule is easier to
//! keep true when it lives in one file.

pub mod crud;
pub mod mappings;
pub mod marketplaces;
pub mod members;
pub mod usage;
