//! The Permissions settings page.
//!
//! It shows the effective rules for the open project and the chosen agent,
//! grouped by permission with where each rule comes from, and the user's own
//! rules added and removed in the global settings file through
//! `harness_config::update_global_setting` on the runtime.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

pub mod jsonc;
mod load;
mod render;
pub mod rules;

use std::path::PathBuf;

use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, SharedString, Subscription, Task, Window,
};
use harness_config::Action;
use harness_ui::components::TextField;
use harness_ui::{IconName, Tone};

use crate::app::{Services, runtime};
use crate::state::AppState;
use crate::strings::settings as text;
use crate::views::settings::permissions::load::{Request, RulesView};
use crate::views::settings::permissions::rules::RuleEdit;
use crate::views::settings::{SettingsPage, reveal};

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "permissions",
        title: text::PERMISSIONS,
        icon: IconName::Shield,
        build,
    }
}

fn build(app: Entity<AppState>, services: Services, window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| PermissionsPage::new(app, services, window, cx))
        .into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scope {
    directory: PathBuf,
    agent: Option<String>,
}

enum Loading {
    Idle,
    Busy,
    Failed,
}

struct PermissionsPage {
    app: Entity<AppState>,
    services: Services,
    scope: Option<Scope>,
    rules: Option<RulesView>,
    loading: Loading,
    field: Entity<TextField>,
    pattern: String,
    permission: SharedString,
    action: Action,
    load_task: Option<Task<()>>,
    tasks: Vec<Task<()>>,
    _observation: Subscription,
}

impl PermissionsPage {
    fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let changed = cx.weak_entity();
        let submitted = cx.weak_entity();
        let field = cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(text::PATTERN)
                .on_change(move |value, _, cx| {
                    let pattern = value.to_string();
                    if changed
                        .update(cx, |page, _| page.pattern = pattern)
                        .is_err()
                    {
                        tracing::trace!("permissions page closed before its pattern changed");
                    }
                })
                .on_submit(move |_, window, cx| {
                    let page = submitted.clone();
                    window.defer(cx, move |window, cx| {
                        if page
                            .update(cx, |page, cx| page.add_rule(window, cx))
                            .is_err()
                        {
                            tracing::trace!("permissions page closed before its rule was added");
                        }
                    });
                })
        });
        let observation = cx.observe(&app, |page: &mut Self, _, cx| page.follow_scope(cx));
        let mut page = Self {
            app,
            services,
            scope: None,
            rules: None,
            loading: Loading::Idle,
            field,
            pattern: String::new(),
            permission: SharedString::new_static(harness_config::permission::SHELL),
            action: Action::Ask,
            load_task: None,
            tasks: Vec::new(),
            _observation: observation,
        };
        page.scope = page.current_scope(cx);
        page.reload(cx);
        page
    }

    fn current_scope(&self, cx: &App) -> Option<Scope> {
        let state = self.app.read(cx);
        let directory = state.project.as_ref()?.directory.clone();
        Some(Scope {
            directory,
            agent: state.agent.clone(),
        })
    }

    fn follow_scope(&mut self, cx: &mut Context<Self>) {
        let scope = self.current_scope(cx);
        if scope != self.scope {
            self.scope = scope;
            self.reload(cx);
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(scope) = self.scope.clone() else {
            self.rules = None;
            self.loading = Loading::Idle;
            cx.notify();
            return;
        };
        let request = Request {
            engine: self.services.engine().clone(),
            paths: self.services.paths().engine_paths(),
            layout: self.services.layout().clone(),
            directory: scope.directory,
            agent: scope.agent,
        };
        self.loading = Loading::Busy;
        let task = runtime::spawn(
            cx,
            self.services.runtime(),
            load::load(request),
            |page, result, cx| {
                match result {
                    Ok(rules) => {
                        page.rules = Some(rules);
                        page.loading = Loading::Idle;
                    },
                    Err(error) => {
                        tracing::warn!(error = %error, "permission rules not loaded");
                        page.loading = Loading::Failed;
                    },
                }
                cx.notify();
            },
        );
        self.load_task = Some(task);
        cx.notify();
    }

    fn add_rule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(rules) = &self.rules else { return };
        if !rules.user.editable {
            return;
        }
        let Some(edit) = rules::add_edit(&rules.user, &self.permission, &self.pattern, self.action)
        else {
            self.app.update(cx, |_, cx| {
                AppState::notify_user(Tone::Warning, text::RULE_INVALID, cx);
            });
            return;
        };
        self.pattern.clear();
        self.field.update(cx, |field, cx| {
            field.set_text(SharedString::default(), window, cx);
        });
        self.apply(edit, cx);
    }

    fn apply(&mut self, edit: RuleEdit, cx: &Context<Self>) {
        let paths = self.services.paths().engine_paths();
        let layout = self.services.layout().clone();
        let write = move || {
            let keys: Vec<&str> = edit.key.iter().map(String::as_str).collect();
            let value = edit.value.as_ref().map(rules::to_json);
            harness_config::update_global_setting(&paths, &layout, &keys, value.as_ref())
        };
        let task = runtime::blocking(cx, self.services.runtime(), write, |page, result, cx| {
            if let Err(error) = result {
                tracing::warn!(error = %error, "permission rule not saved");
                page.app.update(cx, |_, cx| {
                    AppState::notify_user(Tone::Danger, text::RULE_SAVE_FAILED, cx);
                });
            }
            page.reload(cx);
        });
        reveal::keep(&mut self.tasks, task);
    }
}
