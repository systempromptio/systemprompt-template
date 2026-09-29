//! Publication decisions: the manual review form and withdrawal proposals,
//! both decided from a marketplace's Distribution view. The row and tile
//! shaping in [`view`] is shared with that view.

mod review;
pub(crate) mod view;

pub(crate) use review::{decide_withdrawal, review};
