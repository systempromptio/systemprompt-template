//! Persistence for projects: the rows, their membership, and the AD groups
//! that map onto them.
//!
//! Deliberately parallel to [`super::groups`] and not shared with it: a
//! project has no system row and no derived membership, so the two only look
//! alike at the surface.

pub mod activity;
pub mod crud;
pub mod mappings;
pub mod members;
pub mod usage;
