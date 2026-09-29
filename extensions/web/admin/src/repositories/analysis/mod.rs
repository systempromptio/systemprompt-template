//! Analysis repositories.
//!
//! Conversations and skills read the deterministic record — the
//! `conversation_facts` rollup, the hook plane, the tool ledger and the
//! governance spine — with the judge's one label joined on top; tools and
//! artifacts read the ledger through `tool_activity`; reports hold the
//! on-demand AI digests and findings; marketplace versions read the
//! per-skill half of the same rollup (`conversation_skill_facts`), so a
//! version's figures survive raw-event retention. Plugin evaluation scores
//! each of those conversations from its stored messages by fixed rules.
pub mod conversations;
pub mod inventory_index;
pub mod marketplace_versions;
pub mod plugin_eval;
pub mod publications;
pub mod reports;
pub mod skills;
pub mod tools;
