//! Every user-facing string of the Brain's views.
//!
//! The screen, the settings pane and the transcript adornment, in English.
//! Views read their text from here; nothing user-facing is written inline in a
//! view.

use chrono::{DateTime, Local, Utc};
use sp_protocol::wire::brain::{
    FactStatus, ProposalKind, ProposalStatus, RevisionChange, ScopeLevel, WithheldReason,
};

use crate::model::{Availability, BrainFailure, Decision, FactAction};

pub const EXTENSION_NAME: &str = "Brain";
pub const SCREEN_TITLE: &str = "Brain";
pub const SETTINGS_TITLE: &str = "Brain";

pub const TAB_FACTS: &str = "Facts";
pub const TAB_PROPOSALS: &str = "Proposals";
pub const REFRESH: &str = "Refresh";
pub const LOAD_MORE: &str = "Load more";

pub const FILTERS: &str = "Filters";
pub const KIND_PLACEHOLDER: &str = "Kind, such as decision";
pub const SUBJECT_PLACEHOLDER: &str = "Subject, such as client:acme";
pub const KIND_INVALID: &str = "Not a fact kind";
pub const SUBJECT_INVALID: &str = "Write a subject as kind:id";
pub const APPLY_FILTERS: &str = "Apply";
pub const CLEAR_FILTERS: &str = "Clear";
pub const STATUSES: &str = "Status";

pub const NO_FACTS_TITLE: &str = "No facts yet";
pub const NO_FACTS_BODY: &str = "Facts appear here as your team's work is recorded and confirmed.";
pub const NO_MATCHES: &str = "No facts match these filters";
pub const SELECT_FACT_TITLE: &str = "Choose a fact";
pub const SELECT_FACT_BODY: &str = "Its history and why it is recalled show here.";
pub const MORE_FAILED: &str = "The next page could not be read.";

pub const PINNED: &str = "Pinned";
pub const DETAIL_KIND: &str = "Kind";
pub const DETAIL_SUBJECT: &str = "Subject";
pub const DETAIL_SCOPE: &str = "Shared with";
pub const DETAIL_OBSERVED: &str = "Observed";
pub const DETAIL_CONFIRMED: &str = "Confirmed";
pub const DETAIL_REVISION: &str = "Revision";
pub const DETAIL_FIELDS: &str = "Fields";
pub const NOT_CONFIRMED: &str = "Not yet";

pub const WHY_TITLE: &str = "Why it ranks here";
pub const WHY_BASE: &str = "Base importance";
pub const WHY_SUBJECT: &str = "Subject match";
pub const WHY_WORKFLOW: &str = "Workflow match";
pub const WHY_RECENCY: &str = "Recency";
pub const WHY_USAGE: &str = "Usage";
pub const WHY_TOTAL: &str = "Total";
pub const WHY_RECALLED: &str = "Recall would include this fact in your model calls.";

pub const TIMELINE_TITLE: &str = "Timeline";
pub const NO_TIMELINE: &str = "No revisions recorded.";

pub const DISPUTE_PROMPT: &str = "What is wrong with this fact?";
pub const RETRACT_PROMPT: &str = "Why retract it? A reason is optional.";
pub const REASON_REQUIRED: &str = "Give a reason";
pub const SUBMIT: &str = "Submit";
pub const CANCEL: &str = "Cancel";

pub const PROPOSALS_OPEN: &str = "Waiting for a decision";
pub const PROPOSALS_DECIDED: &str = "Decided";
pub const NO_PROPOSALS_TITLE: &str = "No proposals";
pub const NO_PROPOSALS_BODY: &str =
    "Changes your instance's analyses suggest wait here for a person to approve them.";
pub const SHOW_BODY: &str = "Show details";
pub const HIDE_BODY: &str = "Hide details";

pub const SETTINGS_RECALL: &str = "Recall team memory";
pub const SETTINGS_RECALL_BODY: &str = "Let your instance add relevant team facts to this app's model calls. Off sends every call without recalled memory.";
pub const SETTINGS_UNREADABLE: &str =
    "Your Brain preferences could not be read, so recall is off until you set it again.";
pub const SETTINGS_WRITE_FAILED: &str = "The preference could not be saved.";
pub const SETTINGS_AVAILABILITY: &str = "Availability";

pub const fn status(status: FactStatus) -> &'static str {
    match status {
        FactStatus::Candidate => "Candidate",
        FactStatus::Proposed => "Proposed",
        FactStatus::Confirmed => "Confirmed",
        FactStatus::Disputed => "Disputed",
        FactStatus::Stale => "Stale",
        FactStatus::Superseded => "Superseded",
        FactStatus::Retired => "Retired",
    }
}

pub const fn change(change: RevisionChange) -> &'static str {
    match change {
        RevisionChange::Created => "Created",
        RevisionChange::Revised => "Revised",
        RevisionChange::Merged => "Merged",
        RevisionChange::FieldsEdited => "Fields edited",
        RevisionChange::Confirmed => "Confirmed",
        RevisionChange::Disputed => "Disputed",
        RevisionChange::DisputeResolved => "Dispute resolved",
        RevisionChange::Staled => "Marked stale",
        RevisionChange::Superseded => "Superseded",
        RevisionChange::Retracted => "Retracted",
        RevisionChange::Pinned => "Pinned",
        RevisionChange::Unpinned => "Unpinned",
        RevisionChange::ReviewDue => "Review due",
        RevisionChange::Expired => "Expired",
    }
}

pub const fn scope(level: ScopeLevel) -> &'static str {
    match level {
        ScopeLevel::User => "Only you",
        ScopeLevel::Project => "A project",
        ScopeLevel::Group => "A group",
        ScopeLevel::Org => "Your organisation",
    }
}

pub const fn withheld(reason: WithheldReason) -> &'static str {
    match reason {
        WithheldReason::Scope => "Recall leaves it out: it is not shared with this context.",
        WithheldReason::Source => "Recall leaves it out: its source is not trusted here.",
        WithheldReason::AgentCapped => "Recall leaves it out: the agent's fact limit is reached.",
        WithheldReason::Stale => "Recall leaves it out: it is stale.",
        WithheldReason::ReviewDue => "Recall leaves it out: its review is due.",
        WithheldReason::Disputed => "Recall leaves it out: it is disputed.",
        WithheldReason::Quota => "Recall leaves it out: the quota is used up.",
        WithheldReason::MaxFacts => "Recall leaves it out: the fact limit is reached.",
        WithheldReason::Budget => "Recall leaves it out: the token budget is used up.",
        WithheldReason::LowScore => "Recall leaves it out: it ranks too low.",
        WithheldReason::ContextCap => "Recall leaves it out: the context limit is reached.",
        WithheldReason::UnknownName => "Recall leaves it out: a name in it is unknown.",
        WithheldReason::NoSignal => "Recall leaves it out: nothing in the context points to it.",
        WithheldReason::Unavailable => "Recall leaves it out: recall is unavailable.",
    }
}

pub const fn proposal_kind(kind: ProposalKind) -> &'static str {
    match kind {
        ProposalKind::ConfigDiff => "Configuration change",
        ProposalKind::FactActions => "Fact changes",
    }
}

pub const fn proposal_status(status: ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Open => "Open",
        ProposalStatus::Approved => "Approved",
        ProposalStatus::Rejected => "Rejected",
    }
}

pub const fn action(action: FactAction) -> &'static str {
    match action {
        FactAction::Confirm => "Confirm",
        FactAction::Dispute => "Dispute",
        FactAction::Retract => "Retract",
        FactAction::Pin => "Pin",
        FactAction::Unpin => "Unpin",
    }
}

pub const fn action_done(action: FactAction) -> &'static str {
    match action {
        FactAction::Confirm => "Fact confirmed.",
        FactAction::Dispute => "Dispute opened.",
        FactAction::Retract => "Fact retracted.",
        FactAction::Pin => "Fact pinned.",
        FactAction::Unpin => "Fact unpinned.",
    }
}

pub const fn decision(decision: Decision) -> &'static str {
    match decision {
        Decision::Approve => "Approve",
        Decision::Reject => "Reject",
    }
}

pub const fn decision_done(decision: Decision) -> &'static str {
    match decision {
        Decision::Approve => "Proposal approved.",
        Decision::Reject => "Proposal rejected.",
    }
}

pub const fn failure(failure: BrainFailure) -> &'static str {
    match failure {
        BrainFailure::NoInstance => "No instance with the Brain is connected.",
        BrainFailure::Unconfirmed => {
            "The instance has not confirmed its Brain yet. Try again shortly."
        },
        BrainFailure::Disabled => "The Brain is turned off on this instance.",
        BrainFailure::NotFound => "That fact or proposal is no longer visible to you.",
        BrainFailure::Forbidden => "Your account may not do that on this instance.",
        BrainFailure::Tombstoned => "That fact was erased and cannot be recorded again.",
        BrainFailure::Conflict => "That fact changed meanwhile. Refresh and try again.",
        BrainFailure::Invalid => "The instance did not accept that request.",
        BrainFailure::SignedOut => "Sign in again to reach the Brain.",
        BrainFailure::RateLimited => "The instance is busy. Try again in a moment.",
        BrainFailure::Unreachable => "The instance could not be reached.",
        BrainFailure::Unexpected => "The instance answered in a way this app does not understand.",
    }
}

pub const fn availability_title(availability: Availability) -> &'static str {
    match availability {
        Availability::Withheld => "The Brain is not enabled for this app",
        Availability::Detached => "Not connected",
        Availability::Checking => "Checking the Brain",
        Availability::Disabled => "The Brain is turned off",
        Availability::Ready => "Ready",
    }
}

pub const fn availability_body(availability: Availability) -> &'static str {
    match availability {
        Availability::Withheld => "Your instance decides which features this app may use.",
        Availability::Detached => "Sign in to an instance to see your team's memory.",
        Availability::Checking => "Asking your instance whether its Brain is available.",
        Availability::Disabled => "Your instance has its Brain turned off.",
        Availability::Ready => "Your team's memory is available.",
    }
}

pub fn recalled(count: usize) -> String {
    if count == 1 {
        "1 team fact recalled".to_owned()
    } else {
        format!("{count} team facts recalled")
    }
}

pub fn open_proposals(count: usize) -> String {
    format!("{TAB_PROPOSALS} ({count})")
}

pub fn fact_line(kind: &str, subject: &str) -> String {
    format!("{kind} · {subject}")
}

pub fn revision_line(revision: i32, change: RevisionChange) -> String {
    format!("{DETAIL_REVISION} {revision} · {}", self::change(change))
}

pub fn proposal_line(kind: &str, analysis: &str, when: &str) -> String {
    format!("{kind} · {analysis} · {when}")
}

pub fn revision(revision: i32) -> String {
    format!("{DETAIL_REVISION} {revision}")
}

pub fn when(at: &DateTime<Utc>) -> String {
    at.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
