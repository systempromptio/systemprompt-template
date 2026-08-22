//! The vocabulary a diff is reported in: one [`Change`], its [`ChangeKind`],
//! and the [`ChangeSet`] they accumulate into.
//!
//! `Display` is implemented here because the CLI prints these verbatim.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Update,
    Add,
    Remove,
    // Why: Deployed on every apply because it cannot be read back to compare.
    AlwaysApplied,
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Update => "update",
            Self::Add => "add",
            Self::Remove => "remove",
            Self::AlwaysApplied => "always-applied",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub kind: ChangeKind,
    pub path: String,
    pub actual: String,
    pub desired: String,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ChangeKind::AlwaysApplied => {
                write!(f, "  {} {} = {}", self.kind, self.path, self.desired)
            },
            _ => write!(
                f,
                "  {} {}: {} -> {}",
                self.kind, self.path, self.actual, self.desired
            ),
        }
    }
}

/// The full result of comparing two specs.
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    pub changes: Vec<Change>,
}

impl ChangeSet {
    #[must_use]
    pub fn drift(&self) -> Vec<&Change> {
        self.changes
            .iter()
            .filter(|c| c.kind != ChangeKind::AlwaysApplied)
            .collect()
    }

    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.drift().is_empty()
    }
}

pub(super) fn push(
    out: &mut Vec<Change>,
    kind: ChangeKind,
    path: &str,
    actual: &str,
    desired: &str,
) {
    out.push(Change {
        kind,
        path: path.to_owned(),
        actual: actual.to_owned(),
        desired: desired.to_owned(),
    });
}

pub(super) fn compare_str(out: &mut Vec<Change>, path: &str, actual: &str, desired: &str) {
    if actual != desired {
        push(out, ChangeKind::Update, path, actual, desired);
    }
}
