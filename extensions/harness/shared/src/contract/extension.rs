//! The trait a shell extension implements.

use std::rc::Rc;

use systemprompt_engine::run::extension::ExtensionMetadata;

use crate::contract::context::ShellContext;
use crate::contract::contribution::{
    ScreenSpec, SettingsPaneSpec, TitlebarItemSpec, TranscriptAdornment,
};

/// A feature's part of the shell.
///
/// `Default`-constructed from its
/// [`ShellExtensionRegistration`](crate::contract::ShellExtensionRegistration)
/// and held as `dyn ShellExtension`. Only `metadata` is required; its id is the
/// id of the extension's instance-side half, and the instance's enablement of
/// that id decides whether anything here is shown. Contributions are read on
/// the UI thread whenever the shell rebuilds its navigation.
pub trait ShellExtension: Send + Sync + 'static {
    fn metadata(&self) -> ExtensionMetadata;

    fn screens(&self, _ctx: &ShellContext) -> Vec<ScreenSpec> {
        Vec::new()
    }

    fn titlebar_items(&self, _ctx: &ShellContext) -> Vec<TitlebarItemSpec> {
        Vec::new()
    }

    fn settings_panes(&self, _ctx: &ShellContext) -> Vec<SettingsPaneSpec> {
        Vec::new()
    }

    fn adornments(&self, _ctx: &ShellContext) -> Vec<Rc<dyn TranscriptAdornment>> {
        Vec::new()
    }
}
