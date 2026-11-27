//! Governance in the shell.
//!
//! The titlebar pill showing the active instance and how its sync stands, and
//! its Settings page (instances, key trust, Sync now, sign out and remove, Add
//! instance, synced content, the verdict switch and the heartbeat).
//!
//! Contract (fixed by the shell):
//!
//! - `GovernancePill::new(app: Entity<AppState>, services: Services, window:
//!   &mut Window, cx: &mut Context<GovernancePill>) -> GovernancePill`, mounted
//!   in the titlebar's right-hand slot.
//! - `settings_page() -> SettingsPage` (id `governance`), listed in
//!   `crate::settings::pages()`.
//! - Account state is `AppState::accounts`; the sync engine and governance are
//!   found in `services.handles()`, and a part whose handle is absent is not
//!   rendered; the verdict switch is `UiSettings::verdict_enabled`.
//! - Text comes from `crate::strings::governance`.

mod actions;
mod connect;
pub mod feeds;
pub mod heartbeat;
pub mod model;
mod page;
mod page_render;
mod page_sections;
mod page_sync;
mod page_view;
mod pill;
mod pill_menu;
pub mod pill_model;

use gpui_kit::{AnyView, App, AppContext as _, Entity, Window};
use systemprompt_harness_shared::IconName;

use crate::app::Services;
use crate::governance::page::GovernancePage;
use crate::settings::SettingsPage;
use crate::state::AppState;

pub(crate) use pill::GovernancePill;

pub(crate) const PAGE_ID: &str = "governance";

pub(crate) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: PAGE_ID,
        title: systemprompt_harness_brand::identity().governance_label(),
        icon: IconName::Shield,
        build: build_page,
    }
}

fn build_page(
    app: Entity<AppState>,
    services: Services,
    _window: &mut Window,
    cx: &mut App,
) -> AnyView {
    cx.new(|cx| GovernancePage::new(app, services, cx)).into()
}
