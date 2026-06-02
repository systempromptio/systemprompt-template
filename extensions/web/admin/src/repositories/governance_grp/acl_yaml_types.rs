//! Wire types for the bootstrap department loader.
//!
//! Role/gateway ACL rules are parsed by core (`systemprompt_security::authz::
//! AccessControlConfig`), which owns the rule schema, `entity_match` glob
//! expansion, and `default_included`. The only governance YAML this extension
//! still parses itself is the web-owned `departments` table.

use serde::Deserialize;

/// `departments.yaml`. Its `rules:` key (department-scoped grants, removed from
/// core in 0.12.0) is accepted-but-ignored, so unknown fields are not denied.
#[derive(Debug, Default, Deserialize)]
pub struct DepartmentsDoc {
    #[serde(default)]
    pub departments: Vec<YamlDepartment>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlDepartment {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadReport {
    pub departments_upserted: usize,
    pub rules_upserted: usize,
}
