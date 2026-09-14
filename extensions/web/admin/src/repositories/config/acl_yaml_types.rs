//! Report type for the bootstrap access-control loader.
//!
//! Role/gateway ACL rules are parsed by core (`systemprompt_security::authz::
//! AccessControlConfig`), which owns the rule schema, `entity_match` glob
//! expansion, and `default_included`; this extension parses no governance
//! YAML of its own any more.

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadReport {
    pub rules_upserted: usize,
}
