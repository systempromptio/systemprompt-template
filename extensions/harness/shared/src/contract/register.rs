//! One entry of the named registration list of shell extensions.
//!
//! The composition names its shell extensions: nothing is collected at link
//! time. Each entry carries the extension id of the instance-side half it
//! pairs with in core's registration list, and
//! [`ShellExtensions::from_registrations`](crate::contract::ShellExtensions::from_registrations)
//! builds the extensions from the list.

use std::fmt;
use std::sync::Arc;

use systemprompt_identifiers::engine::ExtensionId;

use crate::contract::extension::ShellExtension;

/// One registered shell extension type: its id and the factory that builds it
/// with `Default`.
#[derive(Clone)]
pub struct ShellExtensionRegistration {
    pub id: ExtensionId,
    pub factory: fn() -> Arc<dyn ShellExtension>,
}

impl ShellExtensionRegistration {
    #[must_use]
    pub const fn of<E: ShellExtension + Default>(id: ExtensionId) -> Self {
        Self {
            id,
            factory: build::<E>,
        }
    }
}

fn build<E: ShellExtension + Default>() -> Arc<dyn ShellExtension> {
    Arc::new(E::default())
}

impl fmt::Debug for ShellExtensionRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellExtensionRegistration")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
