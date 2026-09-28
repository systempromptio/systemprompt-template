//! The database-only rule rows of the access-control drift: where each came
//! from, and whether an overwrite from code retires it.

use serde::Serialize;
use systemprompt_security::authz::DASHBOARD_SOURCE;

use super::drift::BandRuleRow;

/// Where a database-only row came from, as far as its `source` column says.
///
/// `Console` was written from the console and stamped as such; `Bundle` was
/// written by a kit's own `access:` block; `Code` is stamped `yaml` — this
/// file wrote it and has since stopped declaring it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrphanOrigin {
    Console,
    Bundle,
    Code,
}

impl OrphanOrigin {
    #[must_use]
    pub fn of_source(source: &str) -> Self {
        if source == DASHBOARD_SOURCE {
            Self::Console
        } else if source.starts_with("bundle:") {
            Self::Bundle
        } else {
            Self::Code
        }
    }
}

/// A database-only rule row.
///
/// `governed` says whether the file still names this entity; `retire` says
/// whether an overwrite deletes the row: every orphan on a governed entity,
/// and every code-written orphan wherever it sits — code removed it, so code
/// takes it away. Console- and bundle-written rows on entities the file does
/// not name are the only orphans it keeps.
#[derive(Debug, Clone, Serialize)]
pub struct OrphanRule {
    pub row: BandRuleRow,
    pub origin: OrphanOrigin,
    pub governed: bool,
    pub retire: bool,
}
