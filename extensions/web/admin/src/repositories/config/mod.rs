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
//! rules are declared in `services/access-control/rules.yaml` — read here,
//! seeded once at boot by `repositories::sync::boot`, and otherwise only
//! compared against the database. Inbound Slack apps are the one projection
//! that stays with the file it gates ([`slack_acl`]). The exception is
//! [`acl_detect`], which is DB-backed but belongs to this domain: it re-runs
//! the configured ACL over traffic that already went through.

pub mod acl_detect;
pub mod agents;
pub mod gateway;
pub mod gateway_acl;
pub mod groups_yaml_loader;
pub mod groups_yaml_types;
pub mod rules_yaml_loader;
pub mod rules_yaml_types;
pub mod slack_acl;
