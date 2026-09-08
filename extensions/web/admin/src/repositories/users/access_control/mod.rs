//! Access-control rule storage and per-user matrix resolution.
//!
//! `rules` owns the CRUD over `access_control_rules`; `matrix` resolves the
//! effective grant for every catalog entity against a single user's rule
//! chain, and `matrix_subject` does the same for a subject that is not a
//! person — a group or a role, as the audience matrix reads them.

pub(crate) mod matrix;
mod matrix_source;
pub mod matrix_subject;
mod matrix_types;
mod rules;

pub use matrix::{
    MatrixRow, MatrixSection, MatrixSource, SectionInput, UserMatrix, UserMatrixUser,
    filter_catalog_for_user, resolve_user_matrix,
};
pub use matrix_subject::{
    MatrixSubject, group_subject, resolve_subject_matrices, resolve_subject_matrix, role_subject,
    user_subject,
};
pub use rules::{
    bulk_set_rules, count_assignments_by_entity_type, list_all_rules, list_rules_for_entity,
    set_entity_rules,
};
