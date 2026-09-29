//! The Analysis section: what the deterministic record says about the
//! instance's conversations, skills and marketplace versions — who, which
//! client and model, tokens, cost, latency, tools, governance — with the
//! judge's one label per conversation joined on top.
//!
//! Every page lives at `/admin/analysis/<noun>` with the same identity in
//! the path as the Platform page for the same noun (`analysis_urls`).
pub(crate) mod conversation_detail;
pub(crate) mod conversations;
pub(crate) mod help;
pub(crate) mod inventory_actions;
pub(crate) mod judge_mode;
pub(crate) mod lifecycle;
pub(crate) mod marketplace_versions;
pub(crate) mod reports;
pub(crate) mod revisions;
pub(crate) mod ribbon;
pub(crate) mod skill_detail;
pub(crate) mod skill_runs;
pub(crate) mod skills;
pub(crate) mod time;
pub(crate) mod tone;
pub(crate) use skill_detail::skill_page;
pub use skill_detail::{SkillRef, parse_skill_key};
