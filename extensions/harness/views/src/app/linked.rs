//! The extensions this build links, each named once.
//!
//! Naming every linked extension type here lets startup report a linked
//! extension that a registration list left out.

use systemprompt_harness::extension::SpExtension;
use systemprompt_harness_shared::contract::{ExtensionMetadata, ShellExtension};

pub(crate) fn instance_side() -> [ExtensionMetadata; 5] {
    [
        describe_instance_side::<systemprompt_harness::governance::GovernanceExtension>(),
        describe_instance_side::<systemprompt_harness::members::drive::DriveExtension>(),
        describe_instance_side::<
            systemprompt_harness::members::notifications::NotificationsExtension,
        >(),
        describe_instance_side::<systemprompt_harness::members::brain::BrainExtension>(),
        describe_instance_side::<systemprompt_harness::members::slack::SlackExtension>(),
    ]
}

pub(crate) fn shell_side() -> [ExtensionMetadata; 4] {
    [
        describe_shell_side::<crate::drive::DriveShellExtension>(),
        describe_shell_side::<crate::notifications::NotificationsShell>(),
        describe_shell_side::<crate::brain::BrainShellExtension>(),
        describe_shell_side::<crate::notifications::SlackShell>(),
    ]
}

pub(crate) fn report_missing<'a>(
    linked: &[ExtensionMetadata],
    discovered: impl IntoIterator<Item = &'a ExtensionMetadata>,
    side: &'static str,
) {
    let discovered: Vec<_> = discovered.into_iter().map(|found| &found.id).collect();
    for metadata in linked {
        if !discovered.contains(&&metadata.id) {
            tracing::error!(extension = %metadata.id, side, "a linked extension was not discovered");
        }
    }
}

fn describe_instance_side<T: SpExtension + Default>() -> ExtensionMetadata {
    let extension: Box<dyn SpExtension> = Box::new(T::default());
    extension.metadata()
}

fn describe_shell_side<T: ShellExtension + Default>() -> ExtensionMetadata {
    let extension: Box<dyn ShellExtension> = Box::new(T::default());
    extension.metadata()
}
