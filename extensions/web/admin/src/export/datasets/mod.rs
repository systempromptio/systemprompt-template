//! One file per exportable table. Each declares its columns, maps a
//! repository row to typed cells, and reads through the page module's own
//! `export_rows`, so the export never drifts from the page.

pub(crate) mod analysis_conversation;
pub(crate) mod analysis_conversations;
pub(crate) mod analysis_skill_conversations;
pub(crate) mod analysis_skill_runs;
pub(crate) mod analysis_skills;
pub(crate) mod contexts;
pub(crate) mod dashboard;
pub(crate) mod dashboard_cost;
pub(crate) mod governance;
pub(crate) mod history;
pub(crate) mod people;
pub(crate) mod plugin_eval;
pub(crate) mod projects;
pub(crate) mod reports;
pub(crate) mod requests;
pub(crate) mod sessions;
pub(crate) mod tools;
pub(crate) mod traces;
pub(crate) mod versions;
