//! Link-time registration of shell extensions over `inventory`.

use std::fmt;
use std::sync::Arc;

use crate::extension::ShellExtension;

/// One registered shell extension type: the factory that builds it with
/// `Default`.
#[derive(Clone, Copy)]
pub struct ShellExtensionRegistration {
    pub factory: fn() -> Arc<dyn ShellExtension>,
}

impl fmt::Debug for ShellExtensionRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellExtensionRegistration")
            .finish_non_exhaustive()
    }
}

inventory::collect!(ShellExtensionRegistration);

#[macro_export]
macro_rules! register_shell_extension {
    ($extension:ty) => {
        $crate::__inventory::submit! {
            $crate::ShellExtensionRegistration {
                factory: || {
                    ::std::sync::Arc::new(<$extension as ::core::default::Default>::default())
                        as ::std::sync::Arc<dyn $crate::ShellExtension>
                },
            }
        }
    };
}
