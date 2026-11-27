//! [`BrainShellExtension`]: the brain's shell half, registered with
//! `register_shell_extension!` under the id of its instance-side half.
//!
//! Every contribution is built over the [`BrainService`] that half publishes in
//! the shared handles; without it nothing is contributed. The shell shows the
//! contributions only while the instance admits the extension.

use std::rc::Rc;

use gpui_kit::{AnyView, AppContext as _};
use harness_shell::{
    ExtensionMetadata, Placement, ScreenSpec, SettingsPaneSpec, ShellContext, ShellExtension,
    TranscriptAdornment, register_shell_extension,
};
use harness_ui::IconName;
use sp_brain::{BRAIN_EXTENSION, BrainService};

use crate::strings;
use crate::view::{BrainScreen, BrainSettingsPane, RecalledFacts};

const SCREEN_KEY: &str = "facts";
const SETTINGS_KEY: &str = "brain";

/// The brain's screens, settings pane and transcript adornment.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrainShellExtension;

register_shell_extension!(BrainShellExtension);

impl ShellExtension for BrainShellExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata::new(BRAIN_EXTENSION, strings::EXTENSION_NAME)
    }

    fn screens(&self, ctx: &ShellContext) -> Vec<ScreenSpec> {
        let Some(service) = service(ctx) else {
            return Vec::new();
        };
        vec![ScreenSpec {
            key: SCREEN_KEY,
            title: strings::SCREEN_TITLE,
            icon: IconName::Sparkle,
            placement: Placement::Sidebar,
            build: Rc::new(move |ctx, window, cx| {
                let service = service.clone();
                AnyView::from(cx.new(|cx| BrainScreen::new(ctx, service, window, cx)))
            }),
        }]
    }

    fn settings_panes(&self, ctx: &ShellContext) -> Vec<SettingsPaneSpec> {
        let Some(service) = service(ctx) else {
            return Vec::new();
        };
        vec![SettingsPaneSpec {
            key: SETTINGS_KEY,
            title: strings::SETTINGS_TITLE,
            icon: IconName::Sparkle,
            build: Rc::new(move |ctx, _window, cx| {
                let service = service.clone();
                AnyView::from(cx.new(|cx| BrainSettingsPane::new(ctx, service, cx)))
            }),
        }]
    }

    fn adornments(&self, ctx: &ShellContext) -> Vec<Rc<dyn TranscriptAdornment>> {
        service(ctx)
            .map(|service| {
                let adornment: Rc<dyn TranscriptAdornment> = Rc::new(RecalledFacts::new(service));
                adornment
            })
            .into_iter()
            .collect()
    }
}

fn service(ctx: &ShellContext) -> Option<BrainService> {
    let found = ctx.handles.get::<BrainService>();
    if found.is_none() {
        tracing::debug!(extension = %BRAIN_EXTENSION, "no brain service is provided; nothing is contributed");
    }
    found.map(|service| BrainService::clone(&service))
}
