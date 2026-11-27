//! How the app state follows the accounts, the engine events, the store
//! changes of the open project and the instance's extension enablement.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::rc::Rc;

use crate::state::app_state::{AppState, Route};
use crate::state::extensions::Contributions;
use gpui_kit::Context;
use harness_engine::{EngineEvent, RunStatus};
use harness_store::Change;
use harness_ui::Tone;
use sp_accounts::AccountsSnapshot;
use sp_extension::EnabledExtensions;

impl AppState {
    pub(super) fn on_accounts(&mut self, snapshot: AccountsSnapshot, cx: &mut Context<Self>) {
        let was_gated = self.needs_sign_in();
        self.accounts = snapshot;
        if was_gated != self.needs_sign_in() {
            self.reload_models(cx);
        }
        cx.notify();
    }

    pub(super) fn on_enablement(
        &mut self,
        enabled: Option<EnabledExtensions>,
        cx: &mut Context<Self>,
    ) {
        if self.enabled == enabled {
            return;
        }
        let services = &self.services;
        self.contributions = Rc::new(Contributions::read(
            services.shell_extensions(),
            enabled.as_ref(),
            services.shell(),
        ));
        self.enabled = enabled;
        if let Route::Screen(key) = &self.route
            && self.contributions.screen(key).is_none()
        {
            self.route = Route::Session;
        }
        cx.notify();
    }

    pub(super) fn on_lag(&mut self, _missed: u64, cx: &mut Context<Self>) {
        if let Some(directory) = self
            .project
            .as_ref()
            .map(|project| project.directory.clone())
        {
            self.load_project(directory, cx);
        }
        self.reload_projects(cx);
    }

    pub(super) fn on_event(&mut self, event: EngineEvent, cx: &mut Context<Self>) {
        match event {
            EngineEvent::Status { session, status } => {
                if matches!(status, RunStatus::Idle) {
                    self.statuses.remove(&session);
                } else {
                    self.statuses.insert(session, status);
                }
                cx.notify();
            },
            EngineEvent::ModelsChanged => self.reload_models(cx),
            EngineEvent::SessionError { session, error }
                if self.current.as_ref() != Some(&session) =>
            {
                let message = crate::strings::foundation::failure(error.kind);
                Self::notify_user(Tone::Danger, message, cx);
            },
            _ => {},
        }
    }

    pub(super) fn on_changes(&mut self, changes: Vec<Change>, cx: &mut Context<Self>) {
        let mut dirty = false;
        for change in changes {
            dirty |= self.apply_change(change);
        }
        if dirty {
            cx.notify();
        }
    }

    pub(super) fn apply_change(&mut self, change: Change) -> bool {
        match change {
            Change::SessionCreated(session) | Change::SessionUpdated(session) => {
                let Some(project) = self.project.as_mut() else {
                    return false;
                };
                if !project.owns(&session) {
                    return false;
                }
                if project.id.is_none() {
                    project.id = Some(session.project_id.clone());
                }
                self.sessions.upsert(*session)
            },
            Change::SessionDeleted { id, .. } => {
                if self.current.as_ref() == Some(&id) {
                    self.current = None;
                }
                self.open.retain(|(open, _)| open != &id);
                self.sessions.remove(&id)
            },
            Change::ProjectUpdated(project) => {
                match self
                    .projects
                    .iter_mut()
                    .find(|known| known.id == project.id)
                {
                    Some(known) => *known = project,
                    None => self.projects.push(project),
                }
                true
            },
            Change::ProjectDeleted { id } => {
                self.projects.retain(|known| known.id != id);
                true
            },
            _ => false,
        }
    }
}
