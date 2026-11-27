use std::sync::Arc;

use harness_drive_ui::{DRIVE_SCREEN, DRIVE_SETTINGS_PANE, DriveShellExtension};
use harness_shell::{ExtensionId, ShellExtension, ShellExtensions};
use sp_drive::DriveExtension;

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
fn the_shell_half_resolves_on_its_own() {
    let extension: Arc<dyn ShellExtension> = Arc::new(DriveShellExtension);
    let extensions = ShellExtensions::from_extensions([extension]).expect("one extension resolves");
    assert_eq!(extensions.entries().len(), 1);
    assert_eq!(extensions.entries()[0].metadata.id, DriveExtension::ID);
}
