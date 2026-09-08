//! Reads for the access-control page: the rule ledger and where each rule
//! came from.
//!
//! The rule CRUD lives in [`crate::repositories::users::access_control`] and
//! stays there — this module only reads. It exists because the page needs one
//! thing that table cannot answer on its own: a rule row carries no
//! provenance, so [`yaml_declared`] rebuilds the set the bootstrap loaders
//! would write from the YAML on disk and the ledger reports every rule
//! outside it as an edit made in this instance.

pub mod rules;
pub mod yaml_declared;
