//! Persistence for the configured policy surface — what an operator has
//! declared may happen.
//!
//! Split out of `governance`, which now owns only the record of what actually
//! did happen. The two answer different questions and change for different
//! reasons: an edit here is an operator changing intent, an insert there is
//! the enforcement path recording an outcome.
//!
//! Most of this is not Postgres at all. The gateway routes live in the
//! profile YAML, agent definitions in `services/agents/`, and access-control
//! rules are bootstrapped from `services/access-control/*.yaml`. The
//! exception is [`acl_detect`], which is DB-backed but belongs to this domain:
//! it re-runs the configured ACL over traffic that already went through.

pub mod acl_detect;
pub mod acl_yaml_loader;
pub mod acl_yaml_snapshot;
pub mod acl_yaml_types;
pub mod agents;
pub mod gateway;
pub mod gateway_acl;
