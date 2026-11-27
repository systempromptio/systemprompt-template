//! The extension crates this build links, each named once.
//!
//! An extension registers itself with a link-time static, and the linker may
//! leave out a crate nothing references, taking its registration with it.
//! Naming every linked extension type here keeps each crate in the binary, and
//! lets startup report a linked extension that discovery did not find.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use harness_shell::{ExtensionMetadata, ShellExtension};
use sp_extension::SpExtension;

pub(crate) fn instance_side() -> [ExtensionMetadata; 5] {
    [
        describe_instance_side::<sp_governance::GovernanceExtension>(),
        describe_instance_side::<sp_drive::DriveExtension>(),
        describe_instance_side::<sp_notifications::NotificationsExtension>(),
        describe_instance_side::<sp_brain::BrainExtension>(),
        describe_instance_side::<sp_slack::SlackExtension>(),
    ]
}

pub(crate) fn shell_side() -> [ExtensionMetadata; 4] {
    [
        describe_shell_side::<harness_drive_ui::DriveShellExtension>(),
        describe_shell_side::<harness_notifications_ui::NotificationsShell>(),
        describe_shell_side::<harness_brain_ui::BrainShellExtension>(),
        describe_shell_side::<harness_notifications_ui::SlackShell>(),
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
