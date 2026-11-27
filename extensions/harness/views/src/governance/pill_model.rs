//! What the titlebar pill shows: the active instance, how its sync stands as an
//! icon and a tone, the suggested fix when it needs attention, and the other
//! signed-in instances it can switch to.

use chrono::{DateTime, Utc};
use systemprompt_device::accounts::AccountsSnapshot;
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::{AccountStatus, SyncErrorKind, SyncStatus};
use systemprompt_harness_shared::{IconName, Tone};

use crate::governance::model::{Ago, Fix, ago, display_name, fix};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PillStatus {
    SignedOut,
    KeyPending { rotated: bool },
    Failed(SyncErrorKind),
    Syncing,
    Synced(Ago),
    Waiting,
    NoSync,
}

impl PillStatus {
    pub const fn icon(&self) -> IconName {
        match self {
            Self::SignedOut | Self::Waiting | Self::NoSync => IconName::Shield,
            Self::KeyPending { .. } => IconName::Key,
            Self::Failed(_) => IconName::Alert,
            Self::Syncing => IconName::Sync,
            Self::Synced(_) => IconName::CircleCheck,
        }
    }

    pub const fn tone(&self) -> Tone {
        match self {
            Self::SignedOut | Self::KeyPending { .. } => Tone::Warning,
            Self::Failed(_) => Tone::Danger,
            Self::Syncing => Tone::Accent,
            Self::Synced(_) => Tone::Success,
            Self::Waiting | Self::NoSync => Tone::Neutral,
        }
    }

    pub const fn pulse(&self) -> bool {
        matches!(self, Self::Syncing)
    }

    pub const fn fix(&self) -> Option<Fix> {
        match self {
            Self::KeyPending { .. } => Some(Fix::ReviewKey),
            Self::Failed(kind) => Some(fix(*kind)),
            _ => None,
        }
    }

    pub const fn needs_attention(&self) -> bool {
        matches!(
            self,
            Self::SignedOut | Self::KeyPending { .. } | Self::Failed(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchTarget {
    pub id: AccountId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillModel {
    pub label: String,
    pub status: PillStatus,
    pub others: Vec<SwitchTarget>,
    pub can_sync: bool,
}

pub fn pill(
    accounts: &AccountsSnapshot,
    sync: Option<&SyncStatus>,
    now: DateTime<Utc>,
    fallback: &str,
) -> PillModel {
    let others = accounts
        .accounts
        .iter()
        .filter(|account| !account.active && account.status == AccountStatus::SignedIn)
        .map(|account| SwitchTarget {
            id: account.id.clone(),
            name: display_name(account, &accounts.accounts),
        })
        .collect();
    let Some(active) = accounts
        .active_account()
        .filter(|account| account.status == AccountStatus::SignedIn)
    else {
        return PillModel {
            label: fallback.to_owned(),
            status: PillStatus::SignedOut,
            others,
            can_sync: false,
        };
    };
    let pending = accounts
        .offers
        .iter()
        .find(|offer| offer.account == active.id);
    let status = match (pending, sync) {
        (Some(offer), _) => PillStatus::KeyPending {
            rotated: offer.offer.pinned.is_some(),
        },
        (None, None) => PillStatus::NoSync,
        (None, Some(status)) if status.account() != Some(&active.id) => PillStatus::Waiting,
        (None, Some(SyncStatus::Idle)) => PillStatus::Waiting,
        (None, Some(SyncStatus::Syncing { .. })) => PillStatus::Syncing,
        (None, Some(SyncStatus::Synced { at, .. })) => PillStatus::Synced(ago(*at, now)),
        (None, Some(SyncStatus::Error { kind, .. })) => PillStatus::Failed(*kind),
    };
    PillModel {
        label: display_name(active, &accounts.accounts),
        status,
        others,
        can_sync: sync.is_some() && pending.is_none(),
    }
}
