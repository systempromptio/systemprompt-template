//! The `governance_chain` tables: `services/governance/config.yaml` staged
//! in the database.
//!
//! Core builds the policy chain from the file at boot
//! (`systemprompt::security::policy::GovernanceConfig::load`) and reads
//! nothing here. The tables hold the same chain so the sync page can show
//! drift, record what was applied and by whom, and export it back as the
//! file; a change lands in enforcement only after a restart, and every
//! surface that shows this plane says so.

pub mod declared;
pub mod export;
pub mod rows;
