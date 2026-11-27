//! The Drive's shell half: the Drive screen, entered from the titlebar, and
//! the Drive settings pane, each present only while the instance-side half
//! provided its handle (the shell adds them only while the instance admits the
//! extension).

use std::rc::Rc;

use gpui_kit::{AnyView, App, AppContext as _, Window};
use systemprompt_harness::members::drive::{DriveExtension, DriveHandle};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::contract::{
    ExtensionMetadata, Placement, ScreenSpec, SettingsPaneSpec, ShellContext, ShellExtension,
};

use crate::drive::settings_pane::DriveSettingsPane;
use crate::drive::strings as text;
use crate::drive::view::DriveView;

pub const DRIVE_SCREEN: &str = "drive";

pub const DRIVE_SETTINGS_PANE: &str = "drive";

/// The Drive as a shell extension: one titlebar screen and one settings pane,
/// keyed by the id of [`DriveExtension`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DriveShellExtension;

impl ShellExtension for DriveShellExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata::new(DriveExtension::ID, text::DRIVE_NAME)
    }

    fn screens(&self, ctx: &ShellContext) -> Vec<ScreenSpec> {
        if !ctx.handles.contains::<DriveHandle>() {
            return Vec::new();
        }
        vec![ScreenSpec {
            key: DRIVE_SCREEN,
            title: text::DRIVE_TITLE,
            icon: IconName::Cloud,
            placement: Placement::Titlebar,
            build: Rc::new(build_screen),
        }]
    }

    fn settings_panes(&self, ctx: &ShellContext) -> Vec<SettingsPaneSpec> {
        if !ctx.handles.contains::<DriveHandle>() {
            return Vec::new();
        }
        vec![SettingsPaneSpec {
            key: DRIVE_SETTINGS_PANE,
            title: text::settings::TITLE,
            icon: IconName::Folder,
            build: Rc::new(build_settings),
        }]
    }
}

fn drive_handle(ctx: &ShellContext) -> DriveHandle {
    ctx.handles
        .get::<DriveHandle>()
        .map_or_else(DriveHandle::new, |handle| DriveHandle::clone(&handle))
}

fn build_screen(ctx: &ShellContext, window: &mut Window, cx: &mut App) -> AnyView {
    let handle = drive_handle(ctx);
    cx.new(|cx| DriveView::new(handle, ctx, window, cx)).into()
}

fn build_settings(ctx: &ShellContext, _window: &mut Window, cx: &mut App) -> AnyView {
    let handle = drive_handle(ctx);
    cx.new(|cx| DriveSettingsPane::new(handle, ctx, cx)).into()
}
