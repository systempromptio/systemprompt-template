//! Registration and discovery of shell extensions, and what a screen shows of
//! itself.

use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::AppContext as _;
use harness_shell::{
    ExtensionId, ExtensionMetadata, Placement, ScreenSpec, ShellExtension, ShellExtensions,
    extension_id, register_shell_extension,
};
use harness_ui::IconName;
use sp_extension::{ExtensionError, ResolveError};

#[derive(Default)]
struct Registered;

impl ShellExtension for Registered {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata::new(extension_id!("registered-ui"), "registered")
    }
}

register_shell_extension!(Registered);

struct Plain(ExtensionMetadata);

impl ShellExtension for Plain {
    fn metadata(&self) -> ExtensionMetadata {
        self.0.clone()
    }
}

fn plain(id: &str, priority: u32) -> Arc<dyn ShellExtension> {
    Arc::new(Plain(
        ExtensionMetadata::new(ExtensionId::parse(id).unwrap(), "plain").with_priority(priority),
    ))
}

#[test]
fn a_registered_shell_extension_is_discovered() {
    let found = ShellExtensions::discover().unwrap();
    assert!(
        found
            .entries()
            .iter()
            .any(|entry| entry.metadata.id.as_str() == "registered-ui")
    );
}

#[test]
fn a_white_label_ui_shadows_the_stock_one() {
    let resolved =
        ShellExtensions::from_extensions([plain("drive", 100), plain("drive", 400)]).unwrap();
    assert_eq!(resolved.entries().len(), 1);
    assert_eq!(resolved.entries()[0].metadata.priority, 400);
}

#[test]
fn an_unbroken_tie_fails_discovery() {
    let error =
        ShellExtensions::from_extensions([plain("drive", 100), plain("drive", 100)]).unwrap_err();
    assert!(matches!(
        error,
        ExtensionError::Resolve(ResolveError::Ambiguous { priority: 100, .. })
    ));
}

#[test]
fn a_screen_describes_itself_without_its_builder() {
    let screen = ScreenSpec {
        key: "facts",
        title: "Facts",
        icon: IconName::Sparkle,
        placement: Placement::Sidebar,
        build: Rc::new(|_, _, cx| cx.new(|_| gpui_kit::EmptyView).into()),
    };
    let shown = format!("{screen:?}");
    assert!(shown.contains("facts"));
    assert!(shown.contains("Sidebar"));
    assert!(!shown.contains("build"));
}
