//! Template context types for the user pages.

use serde::Serialize;

// Why: one checkbox on the detail page. `source` carries where the membership
// came from, because a directory-sourced one is re-projected at every sign-in
// and unchecking it here would silently come back — so the control is rendered
// disabled rather than offered and then undone.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MembershipChoiceView {
    pub id: String,
    pub name: String,
    pub held: bool,
    pub source: &'static str,
    pub from_directory: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleChoiceView {
    pub id: String,
    pub label: String,
    pub held: bool,
    pub platform_only: bool,
}
