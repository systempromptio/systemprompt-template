use std::sync::Arc;

use systemprompt_harness::members::drive::DriveExtension;
use systemprompt_harness_shared::contract::{ExtensionId, ShellExtension, ShellExtensions};
use systemprompt_harness_views::drive::{DRIVE_SCREEN, DRIVE_SETTINGS_PANE, DriveShellExtension};

use crate::TestResult;

#[test]
fn the_shell_half_shares_the_drive_id_and_is_not_required() {
    let metadata = DriveShellExtension.metadata();
    assert_eq!(metadata.id, DriveExtension::ID);
    assert!(!metadata.required);
    assert_eq!(metadata.dependencies, Vec::<ExtensionId>::new());
    assert_eq!(DRIVE_SCREEN, "drive");
    assert_eq!(DRIVE_SETTINGS_PANE, "drive");
}

#[test]
fn the_shell_half_resolves_on_its_own() -> TestResult {
    let extension: Arc<dyn ShellExtension> = Arc::new(DriveShellExtension);
    let extensions = ShellExtensions::from_extensions([extension])?;
    assert_eq!(extensions.entries().len(), 1);
    assert_eq!(extensions.entries()[0].metadata.id, DriveExtension::ID);
    Ok(())
}
