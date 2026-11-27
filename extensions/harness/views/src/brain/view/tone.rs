//! The design system's tone for each fact status and availability.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use harness_ui::Tone;
use sp_protocol::wire::brain::{FactStatus, ProposalStatus};

use crate::model::Availability;

pub(crate) const fn status(status: FactStatus) -> Tone {
    match status {
        FactStatus::Confirmed => Tone::Success,
        FactStatus::Disputed => Tone::Danger,
        FactStatus::Stale => Tone::Warning,
        FactStatus::Candidate | FactStatus::Proposed => Tone::Info,
        FactStatus::Superseded | FactStatus::Retired => Tone::Neutral,
    }
}

pub(crate) const fn proposal(status: ProposalStatus) -> Tone {
    match status {
        ProposalStatus::Open => Tone::Accent,
        ProposalStatus::Approved => Tone::Success,
        ProposalStatus::Rejected => Tone::Neutral,
    }
}

pub(crate) const fn availability(availability: Availability) -> Tone {
    match availability {
        Availability::Ready => Tone::Success,
        Availability::Checking => Tone::Info,
        Availability::Disabled | Availability::Withheld => Tone::Warning,
        Availability::Detached => Tone::Neutral,
    }
}
