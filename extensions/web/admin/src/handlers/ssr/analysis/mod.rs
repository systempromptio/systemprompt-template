//! The Analysis section: what the deterministic record says about the
//! instance's conversations, skills and marketplace versions — who, which
//! client and model, tokens, cost, latency, tools, governance — with the
//! judge's one label per conversation joined on top.
//!
//! Every page lives at `/admin/analysis/<noun>` with the same identity in
//! the path as the Platform page for the same noun (`analysis_urls`).
#[cfg(feature = "agents")]
pub(crate) mod conversation_detail;
#[cfg(feature = "agents")]
pub(crate) mod conversations;
#[cfg(feature = "agents")]
pub(crate) mod help;
#[cfg(feature = "mcp")]
pub(crate) mod inventory_actions;
#[cfg(feature = "agents")]
pub(crate) mod judge_mode;
#[cfg(feature = "mcp")]
pub(crate) mod lifecycle;
#[cfg(feature = "mcp")]
pub(crate) mod marketplace_versions;
#[cfg(feature = "agents")]
pub(crate) mod reports;
#[cfg(feature = "mcp")]
pub(crate) mod revisions;
#[cfg(feature = "agents")]
pub(crate) mod ribbon;
#[cfg(feature = "agents")]
pub(crate) mod skill_detail;
#[cfg(feature = "agents")]
pub(crate) mod skill_runs;
#[cfg(feature = "agents")]
pub(crate) mod skills;
pub(crate) mod time;
#[cfg(feature = "mcp")]
pub(crate) mod tone;
#[cfg(feature = "agents")]
pub(crate) use skill_detail::skill_page;
#[cfg(feature = "agents")]
pub use skill_detail::{SkillRef, parse_skill_key};
