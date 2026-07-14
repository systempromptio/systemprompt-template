//! `governance_decisions` insert primitive — re-exported from core.
//!
//! The table schema and the canonical writer both live in
//! `systemprompt_security::authz`. This module exists only as a stable import
//! path for template-side callers (the `govern_authz` webhook handler, the
//! tool-use governance handler, `gateway_catalog` inserts) so they can switch
//! to core without churning every import in one PR.

pub use systemprompt_security::authz::{GovernanceDecisionRecord, insert_governance_decision};
