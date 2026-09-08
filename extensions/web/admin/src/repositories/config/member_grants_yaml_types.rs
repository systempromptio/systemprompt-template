//! Wire types for the bootstrap member-grant loaders
//! (`access-control/groups.yaml` and `access-control/projects.yaml`).
//!
//! Each file lists entities confined to members of named groups or projects.
//! Grants default to allow; an explicit deny excludes a group from a shared
//! resource without changing the role grants for other audiences.
//! The two files differ only in the dimension they write at, which is why the
//! loader takes the rule type as a parameter.

use serde::Deserialize;
use systemprompt_security::authz::Access;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberGrantsDoc {
    #[serde(default)]
    pub grants: Vec<MemberGrant>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberGrant {
    #[serde(default = "allow_access")]
    pub access: Access,
    pub entity_type: String,
    pub entity_id: String,
    // Why: the ids of the groups (or projects) whose members reach this
    // entity. Named `members` rather than `groups` so one type serves both
    // files without one of them reading as a lie.
    pub members: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MemberGrantsLoadReport {
    pub grants_projected: usize,
}

const fn allow_access() -> Access {
    Access::Allow
}
