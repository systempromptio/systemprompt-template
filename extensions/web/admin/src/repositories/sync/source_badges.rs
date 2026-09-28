//! The badges on a source row: the one-word state an operator scans for.
//!
//! `in step` is the absence of every warning: pinned digest equals active
//! digest, the composition is reconciled, and the kit carries no access
//! block of its own. A source that follows a channel tag says so as
//! information, not a warning — that is how a kit whose CI publishes on
//! release reaches the instance with one Import press and no re-pin.

use serde::Serialize;

/// A badge on a source row; `tone` is a design-system tone token.
#[derive(Debug, Clone, Serialize)]
pub struct SourceBadge {
    pub text: &'static str,
    pub tone: &'static str,
    pub detail: String,
}

pub(super) struct BadgeInputs<'a> {
    pub name: &'a str,
    pub pinned_digest: Option<&'a str>,
    pub active_digest: Option<&'a str>,
    pub following_tag: bool,
    pub restart_pending: bool,
    pub kit_access: Vec<&'a str>,
}

pub(super) fn source_badges(input: &BadgeInputs<'_>) -> Vec<SourceBadge> {
    let mut badges = Vec::new();
    match (input.pinned_digest, input.active_digest) {
        (Some(p), Some(a)) if p != a => badges.push(SourceBadge {
            text: "pinned ≠ active",
            tone: "warn",
            detail: format!("profile pins {p}; the active tree was fetched as {a}"),
        }),
        (_, None) => badges.push(SourceBadge {
            text: "not fetched",
            tone: "err",
            detail: "no state for this source — it was never fetched, or the fetch failed"
                .to_owned(),
        }),
        _ => {},
    }
    if input.restart_pending && input.active_digest.is_some() {
        badges.push(SourceBadge {
            text: "reconcile pending",
            tone: "warn",
            detail: "the composition was recomposed but not projected into the authz tables; \
                     press Import (Refresh sources) to reconcile it in place"
                .to_owned(),
        });
    }
    let warned = !badges.is_empty();
    if input.following_tag {
        badges.push(SourceBadge {
            text: "channel",
            tone: "info",
            detail: "the profile follows a tag the kit's CI moves on release; Import fetches \
                     whatever the tag points at, and the active digest above is what is served"
                .to_owned(),
        });
    }
    if !input.kit_access.is_empty() {
        badges.push(SourceBadge {
            text: "kit declares access",
            tone: "err",
            detail: format!(
                "the kit's sidecar carries an access block for {} — ignored as a declaration \
                 here; remove it from the kit and declare marketplace/<id> with owner: bundle:{} \
                 in rules.yaml",
                input.kit_access.join(", "),
                input.name
            ),
        });
    }
    if !warned && input.kit_access.is_empty() {
        badges.push(SourceBadge {
            text: "in step",
            tone: "ok",
            detail: "the active digest is what the profile resolves to and the composition is \
                     reconciled"
                .to_owned(),
        });
    }
    badges
}
