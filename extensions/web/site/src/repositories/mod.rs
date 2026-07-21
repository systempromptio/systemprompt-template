//! Database access for the public site.
//!
//! The only place in this crate that touches `sqlx`; providers call these
//! functions rather than issuing queries themselves.

pub(crate) mod blog;
pub(crate) mod docs;
