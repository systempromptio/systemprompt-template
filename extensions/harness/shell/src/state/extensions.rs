//! What the admitted shell extensions contribute (screens, their titlebar
//! items, settings panes and transcript adornments), read again whenever the
//! instance's enablement changes.

use std::fmt;
use std::rc::Rc;

use harness_shell::{
    Contributed, Placement, ScreenKey, ScreenSpec, SettingsPaneSpec, ShellContext, ShellExtensions,
    TitlebarItemSpec, TranscriptAdornment,
};
use sp_extension::EnabledExtensions;

#[derive(Default)]
pub(crate) struct Contributions {
    pub(crate) screens: Vec<Contributed<ScreenSpec>>,
    pub(crate) titlebar_items: Vec<Contributed<TitlebarItemSpec>>,
    pub(crate) settings_panes: Vec<Contributed<SettingsPaneSpec>>,
    pub(crate) adornments: Vec<Contributed<Rc<dyn TranscriptAdornment>>>,
}

impl Contributions {
    pub(crate) fn read(
        extensions: &ShellExtensions,
        enabled: Option<&EnabledExtensions>,
        ctx: &ShellContext,
    ) -> Self {
        Self {
            screens: extensions.screens(enabled, ctx),
            titlebar_items: extensions.titlebar_items(enabled, ctx),
            settings_panes: extensions.settings_panes(enabled, ctx),
            adornments: extensions.adornments(enabled, ctx),
        }
    }

    pub(crate) fn screen(&self, key: &ScreenKey) -> Option<&ScreenSpec> {
        self.screens
            .iter()
            .find(|screen| screen.extension == key.extension && screen.item.key == key.screen)
            .map(|screen| &screen.item)
    }

    pub(crate) fn titlebar_item(&self, key: &ScreenKey) -> Option<&TitlebarItemSpec> {
        self.titlebar_items
            .iter()
            .find(|item| item.extension == key.extension && item.item.screen == key.screen)
            .map(|item| &item.item)
    }

    pub(crate) fn placed(
        &self,
        placement: Placement,
    ) -> impl Iterator<Item = (ScreenKey, &ScreenSpec)> {
        self.screens
            .iter()
            .filter(move |screen| screen.item.placement == placement)
            .map(|screen| {
                let key = ScreenKey {
                    extension: screen.extension.clone(),
                    screen: screen.item.key,
                };
                (key, &screen.item)
            })
    }
}

impl fmt::Debug for Contributions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Contributions")
            .field("screens", &self.screens.len())
            .field("titlebar_items", &self.titlebar_items.len())
            .field("settings_panes", &self.settings_panes.len())
            .field("adornments", &self.adornments.len())
            .finish()
    }
}
