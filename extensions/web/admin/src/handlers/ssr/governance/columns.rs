//! Column headers per tab.
//!
//! Only the decisions log is sortable. The safety and hook logs are ordered by
//! time and nothing else, because a finding read out of time order loses the
//! sequence that makes it legible; a header that offered a sort those queries
//! do not implement would be a lie in the markup.
//!
//! The decisions log no longer offers a sort by policy or by tool either. A row
//! is a whole call now, and a call holds several policies and may touch several
//! targets, so those sorts would order the table by an arbitrary member of each
//! group — the same kind of lie, arrived at from the other direction.

use super::urls::ColumnHeader;
use super::{GovernanceQuery, GovernanceTab};
use crate::repositories::governance::decision_log::DecisionSort;

pub(super) fn columns(
    query: &GovernanceQuery,
    tab: GovernanceTab,
    sort: DecisionSort,
) -> Vec<ColumnHeader> {
    match tab {
        GovernanceTab::Decisions => vec![
            ColumnHeader::sortable("When", "sp-col-date", "when", query, sort),
            ColumnHeader::sortable("Outcome", "sp-col-status", "decision", query, sort),
            ColumnHeader::plain("Target", "sp-col-text"),
            ColumnHeader::sortable("User", "sp-col-identity", "user", query, sort),
            ColumnHeader::plain("Reason", "sp-col-multiline"),
            ColumnHeader::plain("Chain", "sp-col-text"),
            ColumnHeader::plain("Trace", "sp-col-trace"),
        ],
        GovernanceTab::Safety => vec![
            ColumnHeader::plain("When", "sp-col-date"),
            ColumnHeader::plain("Category", "sp-col-text"),
            ColumnHeader::plain("Scanner", "sp-col-text"),
            ColumnHeader::plain("Severity", "sp-col-status"),
            ColumnHeader::plain("Direction", "sp-col-text"),
            ColumnHeader::plain("Outcome", "sp-col-status"),
            ColumnHeader::plain("Model", "sp-col-model"),
            ColumnHeader::plain("Excerpt", "sp-col-multiline"),
            ColumnHeader::plain("Request", "sp-col-trace"),
        ],
        GovernanceTab::Hooks => vec![
            ColumnHeader::plain("When", "sp-col-date"),
            ColumnHeader::plain("Hook", "sp-col-text"),
            ColumnHeader::plain("Tool", "sp-col-text"),
            ColumnHeader::plain("Plugin", "sp-col-text"),
            ColumnHeader::plain("User", "sp-col-identity"),
            ColumnHeader::plain("Status", "sp-col-status"),
        ],
    }
}
