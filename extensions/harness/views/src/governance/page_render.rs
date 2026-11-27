//! The Governance settings page drawn from its model.

use super::page::{GovernancePage, PAGE_WIDTH};
use crate::governance::feeds::{self};
use crate::governance::heartbeat::heartbeat_line;
use crate::governance::model::{Inputs, instances};
use crate::governance::{page_sections, page_view};
use chrono::Utc;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use systemprompt_harness_shared::theme::space;

impl Render for GovernancePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let accounts = self.app.read(cx).accounts.clone();
        let inputs = Inputs {
            accounts: &accounts,
            sync: self.sync.as_ref(),
            last_synced: self.last_synced.as_ref(),
            now: Utc::now(),
        };
        let rows = instances(inputs);
        let active_name = rows
            .iter()
            .find(|row| row.active && row.signed_in)
            .map(|row| row.name.clone());
        let verdict = page_sections::VerdictState {
            enabled: self
                .services
                .settings()
                .read(|settings| settings.verdict_enabled),
            available: feeds::governance(&self.services).is_some(),
            governed: self.held.as_ref().map(|held| held.governed),
        };
        let heartbeat = heartbeat_line(self.heartbeat.as_ref());
        div()
            .id("governance-page")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XL)
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .p(space::XL)
                    .child(page_sections::header(
                        self.services.identity().governance_label(),
                        cx,
                    ))
                    .children((!accounts.secure_storage).then(page_sections::insecure_banner))
                    .child(page_view::instance_list(&rows, cx))
                    .children(
                        active_name
                            .map(|name| page_sections::synced(&name, self.held.as_ref(), cx)),
                    )
                    .child(page_sections::verdict(verdict, cx))
                    .child(page_sections::heartbeat(&heartbeat, cx)),
            )
    }
}
