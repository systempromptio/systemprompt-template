//! The heartbeat line of the Governance page: online, failing, paused, or a
//! version the instance does not accept with the minimum it asks for.

use systemprompt_harness::sync::HeartbeatState;
use systemprompt_harness_shared::Tone;
use systemprompt_models::bridge::instance::WindowStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unsupported {
    TooOld,
    TooNew,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatLine {
    Unavailable,
    Paused,
    Online,
    Failing,
    Unsupported {
        minimum: String,
        reason: Unsupported,
    },
}

impl HeartbeatLine {
    pub const fn tone(&self) -> Tone {
        match self {
            Self::Unavailable | Self::Paused => Tone::Neutral,
            Self::Online => Tone::Success,
            Self::Failing => Tone::Warning,
            Self::Unsupported { .. } => Tone::Danger,
        }
    }
}

pub fn heartbeat_line(state: Option<&HeartbeatState>) -> HeartbeatLine {
    match state {
        None => HeartbeatLine::Unavailable,
        Some(HeartbeatState::Paused) => HeartbeatLine::Paused,
        Some(HeartbeatState::Online) => HeartbeatLine::Online,
        Some(HeartbeatState::Failing) => HeartbeatLine::Failing,
        Some(HeartbeatState::Unsupported {
            min_bridge_version,
            window,
        }) => HeartbeatLine::Unsupported {
            minimum: min_bridge_version.clone(),
            reason: match window {
                Some(WindowStatus::Below) => Unsupported::TooOld,
                Some(WindowStatus::Above) => Unsupported::TooNew,
                Some(WindowStatus::Within | WindowStatus::Unknown) | None => Unsupported::Refused,
            },
        },
    }
}
