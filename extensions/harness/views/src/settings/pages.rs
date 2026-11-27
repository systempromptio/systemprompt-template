//! The pages Settings lists: the shell's own, with the settings panes of the
//! admitted extensions after Governance.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{AnyView, App, Entity, Window};
use harness_shell::SettingsPaneSpec;
use harness_ui::IconName;

use crate::app::Services;
use crate::state::AppState;
use crate::state::extensions::Contributions;
use crate::views::settings::SettingsPage;

const EXTENSIONS_AFTER: &str = "governance";

#[derive(Clone)]
pub(super) enum Page {
    Shell(SettingsPage),
    Extension(SettingsPaneSpec),
}

impl Page {
    pub(super) const fn id(&self) -> &'static str {
        match self {
            Self::Shell(page) => page.id,
            Self::Extension(pane) => pane.key,
        }
    }

    pub(super) const fn title(&self) -> &'static str {
        match self {
            Self::Shell(page) => page.title,
            Self::Extension(pane) => pane.title,
        }
    }

    pub(super) const fn icon(&self) -> IconName {
        match self {
            Self::Shell(page) => page.icon,
            Self::Extension(pane) => pane.icon,
        }
    }

    pub(super) fn build(
        &self,
        app: &Entity<AppState>,
        services: &Services,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyView {
        match self {
            Self::Shell(page) => (page.build)(app.clone(), services.clone(), window, cx),
            Self::Extension(pane) => (pane.build)(services.shell(), window, cx),
        }
    }
}

pub(super) fn pages(contributions: &Contributions) -> Vec<Page> {
    let mut pages: Vec<Page> = super::pages().into_iter().map(Page::Shell).collect();
    let mut at = pages
        .iter()
        .position(|page| page.id() == EXTENSIONS_AFTER)
        .map_or(pages.len(), |index| index + 1);
    for pane in &contributions.settings_panes {
        if pages.iter().any(|page| page.id() == pane.item.key) {
            tracing::warn!(extension = %pane.extension, page = pane.item.key, "a settings pane reuses a page id; not listed");
            continue;
        }
        pages.insert(at, Page::Extension(pane.item.clone()));
        at += 1;
    }
    pages
}
