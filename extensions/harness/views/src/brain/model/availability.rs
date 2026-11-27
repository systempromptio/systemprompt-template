//! Whether the brain can be used from the shell right now.

use sp_brain::{BRAIN_EXTENSION, BrainStatus};
use sp_extension::EnabledExtensions;

/// Where the brain stands for the views.
///
/// `Withheld` while the applied instance has not enabled the extension for
/// this host (or no instance is applied); otherwise the service's status:
/// detached from any instance, checking whether the instance's brain is
/// enabled, disabled there, or ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Availability {
    Withheld,
    Detached,
    Checking,
    Disabled,
    Ready,
}

impl Availability {
    pub fn of(enabled: Option<&EnabledExtensions>, status: BrainStatus) -> Self {
        if !enabled.is_some_and(|enabled| enabled.contains(&BRAIN_EXTENSION)) {
            return Self::Withheld;
        }
        match status {
            BrainStatus::Detached => Self::Detached,
            BrainStatus::Checking => Self::Checking,
            BrainStatus::Disabled => Self::Disabled,
            BrainStatus::Enabled => Self::Ready,
        }
    }

    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}
