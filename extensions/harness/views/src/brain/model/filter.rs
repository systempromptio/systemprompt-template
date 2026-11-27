//! Which facts the list shows: a kind, a subject and any of several statuses,
//! turned into the instance's listing query one page at a time.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_brain::FactQuery;
use sp_protocol::wire::brain::{FactKindId, FactStatus, SubjectKey, SubjectKeyError};
use systemprompt_identifiers::error::IdValidationError;

pub const PAGE_SIZE: u32 = 50;

/// Why a typed filter is not one the instance accepts.
#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("the kind filter is not a fact kind")]
    Kind {
        #[source]
        source: IdValidationError,
    },
    #[error("the subject filter is not a subject key")]
    Subject {
        #[source]
        source: SubjectKeyError,
    },
}

/// The listing filter; statuses are kept in [`FactStatus::ALL`] order and
/// none means any status.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactFilter {
    pub kind: Option<FactKindId>,
    pub subject: Option<SubjectKey>,
    pub statuses: Vec<FactStatus>,
}

impl FactFilter {
    pub fn parse_kind(text: &str) -> Result<Option<FactKindId>, FilterError> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(None);
        }
        FactKindId::try_new(text)
            .map(Some)
            .map_err(|source| FilterError::Kind { source })
    }

    pub fn parse_subject(text: &str) -> Result<Option<SubjectKey>, FilterError> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(None);
        }
        text.parse::<SubjectKey>()
            .map(Some)
            .map_err(|source| FilterError::Subject { source })
    }

    pub fn has_status(&self, status: FactStatus) -> bool {
        self.statuses.contains(&status)
    }

    pub fn toggle_status(&mut self, status: FactStatus) {
        let selected = !self.has_status(status);
        self.statuses = FactStatus::ALL
            .into_iter()
            .filter(|known| {
                if *known == status {
                    selected
                } else {
                    self.statuses.contains(known)
                }
            })
            .collect();
    }

    pub const fn is_empty(&self) -> bool {
        self.kind.is_none() && self.subject.is_none() && self.statuses.is_empty()
    }

    pub fn query(&self, after: Option<String>) -> FactQuery {
        FactQuery {
            kind: self.kind.clone(),
            subject: self.subject.clone(),
            statuses: self.statuses.clone(),
            limit: Some(PAGE_SIZE),
            after,
            ..FactQuery::default()
        }
    }
}
