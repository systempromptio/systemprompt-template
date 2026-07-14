//! Access-control rule storage and per-user matrix resolution.
//!
//! [`rules`] owns the CRUD over `access_control_rules`; [`matrix`] resolves the
//! effective grant for every catalog entity against a single user's rule chain.

mod matrix;
mod rules;

pub use matrix::{
    MatrixRow, MatrixSection, MatrixSource, SectionInput, UserMatrix, UserMatrixUser,
    filter_catalog_for_user, resolve_user_matrix,
};
pub use rules::{
    bulk_set_rules, count_assignments_by_entity_type, list_all_rules, list_rules_for_entity,
    set_entity_rules,
};
