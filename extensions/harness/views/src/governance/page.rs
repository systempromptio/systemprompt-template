//! The Governance settings page entity: it follows the accounts through the app
//! state, the sync status and the heartbeat through their feeds, reloads what
//! the applied instance holds after each sync, and runs the per-instance
//! actions, the confirmations and the Add instance dialog.

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AppContext as _, Context, Entity, Subscription, Task, Window, px};
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::{KeyFingerprint, SyncStatus};
use systemprompt_harness::sync::HeartbeatState;
use systemprompt_harness_shared::Tone;
use systemprompt_harness_shared::components::{ButtonKind, DialogAction, DialogSpec};

use crate::app::{Services, runtime};
use crate::governance::actions;
use crate::governance::connect::{ConnectEvent, ConnectForm};
use crate::governance::feeds::{self, Held};
use crate::governance::model::{InstanceAction, LastSynced, remember_synced};
use crate::state::AppState;
use crate::strings::governance as text;

pub(super) const PAGE_WIDTH: f32 = 760.;
const DIALOG_WIDTH: f32 = 460.;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccountTarget {
    pub(crate) id: AccountId,
    pub(crate) name: String,
    pub(crate) url: String,
}

pub(crate) struct GovernancePage {
    pub(super) app: Entity<AppState>,
    pub(super) services: Services,
    pub(super) sync: Option<SyncStatus>,
    pub(super) heartbeat: Option<HeartbeatState>,
    pub(super) last_synced: Option<LastSynced>,
    pub(super) held: Option<Held>,
    tasks: Vec<Task<()>>,
    connect_events: Option<Subscription>,
    _feeds: Vec<Task<()>>,
    _observation: Subscription,
}

impl std::fmt::Debug for GovernancePage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernancePage")
            .field("sync", &self.sync)
            .finish_non_exhaustive()
    }
}

impl GovernancePage {
    pub(crate) fn new(app: Entity<AppState>, services: Services, cx: &mut Context<Self>) -> Self {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        let mut running = Vec::new();
        let (mut sync, mut heartbeat) = (None, None);
        if let Some(engine) = feeds::sync_engine(&services) {
            sync = Some(engine.status());
            heartbeat = Some(engine.heartbeat());
            running.push(runtime::pump(
                cx,
                engine.subscribe(),
                Self::on_sync,
                |_, _, _| {},
            ));
            running.push(runtime::pump(
                cx,
                engine.subscribe_heartbeat(),
                Self::on_heartbeat,
                |_, _, _| {},
            ));
        }
        let last_synced = sync
            .as_ref()
            .and_then(|status| remember_synced(None, status));
        let mut page = Self {
            app,
            services,
            sync,
            heartbeat,
            last_synced,
            held: None,
            tasks: Vec::new(),
            connect_events: None,
            _feeds: running,
            _observation: observation,
        };
        page.reload_held(cx);
        page
    }

    fn on_sync(&mut self, status: SyncStatus, cx: &mut Context<Self>) {
        self.last_synced = remember_synced(self.last_synced.take(), &status);
        let synced = matches!(status, SyncStatus::Synced { .. } | SyncStatus::Idle);
        self.sync = Some(status);
        if synced {
            self.reload_held(cx);
        }
        cx.notify();
    }

    fn on_heartbeat(&mut self, state: HeartbeatState, cx: &mut Context<Self>) {
        self.heartbeat = Some(state);
        cx.notify();
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.retain(|task| !task.is_ready());
        self.tasks.push(task);
    }

    fn reload_held(&mut self, cx: &Context<Self>) {
        let Some(governance) = feeds::governance(&self.services) else {
            return;
        };
        let engine = self.services.engine().clone();
        let directory = self.app.read(cx).project.as_ref().map_or_else(
            || self.services.paths().home.clone(),
            |project| project.directory.clone(),
        );
        let managed_root = self
            .services
            .paths()
            .config
            .join(systemprompt_harness::projection::SKILLS_DIR);
        let job = feeds::load_held(governance, engine, directory, managed_root);
        let task = runtime::spawn(cx, self.services.runtime(), job, |this, held, cx| {
            this.held = held;
            cx.notify();
        });
        self.keep(task);
    }

    pub(crate) fn run(
        &mut self,
        action: InstanceAction,
        target: &AccountTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            InstanceAction::MakeActive => {
                let task = actions::activate(
                    &self.services,
                    self.app.clone(),
                    (target.id.clone(), target.name.clone()),
                    cx,
                );
                self.keep(task);
            },
            InstanceAction::SyncNow => actions::sync_now(&self.services, &self.app, cx),
            InstanceAction::SignIn => self.open_connect(Some(target.url.clone()), window, cx),
            InstanceAction::SignOut => {
                Self::confirm(target.id.clone(), Confirm::SignOut, window, cx);
            },
            InstanceAction::Remove => Self::confirm(target.id.clone(), Confirm::Remove, window, cx),
            InstanceAction::Trust => {},
        }
    }

    pub(crate) fn trust(&mut self, id: AccountId, fingerprint: KeyFingerprint, cx: &Context<Self>) {
        let task = actions::trust(&self.services, self.app.clone(), (id, fingerprint), cx);
        self.keep(task);
    }

    pub(crate) fn set_verdict(&self, enabled: bool, cx: &mut Context<Self>) {
        self.services
            .settings()
            .update(|settings| settings.verdict_enabled = enabled);
        if let Some(governance) = feeds::governance(&self.services) {
            governance.set_verdicts_enabled(enabled);
        }
        cx.notify();
    }

    fn confirm(id: AccountId, kind: Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let page = cx.entity();
        let (title, body, label) = match kind {
            Confirm::SignOut => (text::SIGN_OUT_TITLE, text::SIGN_OUT_BODY, text::SIGN_OUT),
            Confirm::Remove => (text::REMOVE_TITLE, text::REMOVE_BODY, text::REMOVE),
        };
        let confirm = DialogAction::new(label, move |_, cx| {
            let id = id.clone();
            page.update(cx, |page, cx| page.apply(kind, id, cx));
            true
        })
        .kind(ButtonKind::Danger);
        DialogSpec::new(title)
            .description(body)
            .confirm(confirm)
            .cancel(DialogAction::new(text::CANCEL, |_, _| true))
            .open(window, cx);
    }

    fn apply(&mut self, kind: Confirm, id: AccountId, cx: &Context<Self>) {
        let task = match kind {
            Confirm::SignOut => actions::sign_out(&self.services, self.app.clone(), id, cx),
            Confirm::Remove => actions::remove(&self.services, self.app.clone(), id, cx),
        };
        self.keep(task);
    }

    pub(crate) fn open_connect(
        &mut self,
        url: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let services = self.services.clone();
        let form = cx.new(|cx| ConnectForm::new(services, url, window, cx));
        self.connect_events = Some(cx.subscribe_in(
            &form,
            window,
            |this, _, event: &ConnectEvent, window, cx| {
                this.on_connect(*event, window, cx);
            },
        ));
        let body = form.clone();
        let accounts = self.services.accounts().clone();
        DialogSpec::new(text::CONNECT_TITLE)
            .width(px(DIALOG_WIDTH))
            .body(move |_, _| body.clone())
            .on_dismiss(move |_, _| accounts.cancel_sign_in())
            .open(window, cx);
        form.update(cx, |form, cx| form.focus(window, cx));
    }

    fn on_connect(&mut self, event: ConnectEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            ConnectEvent::SignedIn => {
                window.close_dialog(cx);
                self.connect_events = None;
                actions::notify(&self.app, Tone::Success, text::CONNECT_DONE, cx);
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirm {
    SignOut,
    Remove,
}
