//! What the Drive screen is pointed at: the account and the instance whose
//! Drive the handle follows. A change of either resets what the screen shows.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_protocol::ids::AccountId;

/// The account and instance the Drive screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveTarget {
    account: Option<AccountId>,
    instance_base: String,
}

impl DriveTarget {
    pub fn new(account: Option<AccountId>, instance_base: &str) -> Self {
        Self {
            account,
            instance_base: instance_base.to_owned(),
        }
    }

    pub const fn account(&self) -> Option<&AccountId> {
        self.account.as_ref()
    }

    pub fn instance_base(&self) -> &str {
        &self.instance_base
    }
}
