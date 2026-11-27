//! The titlebar pill: the active instance with an icon and tone for how its
//! sync stands, growing to show the status on hover. A click opens a small
//! popover to fix what needs attention, Sync now, switch instance or open the
//! page.

use chrono::Utc;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button as ToolkitButton, ButtonVariants as _};
use gpui_kit::component::popover::Popover;
use gpui_kit::{
    Anchor, AnyElement, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window,
    div, px,
};
use harness_ui::theme::{size, space, text as type_scale, tokens};
use harness_ui::{Icon, motion};
use sp_protocol::ids::AccountId;
use sp_protocol::view::SyncStatus;

use crate::app::{Services, runtime};
use crate::state::{AppState, Route};
use crate::strings::governance as text;
use crate::views::governance::model::Fix;
use crate::views::governance::pill_model::{PillModel, pill};
use crate::views::governance::{PAGE_ID, actions, feeds};

const LABEL_WIDTH: f32 = 160.;
pub(super) const MENU_WIDTH: f32 = 260.;

pub(crate) struct GovernancePill {
    app: Entity<AppState>,
    services: Services,
    sync: Option<SyncStatus>,
    hovered: bool,
    open: bool,
    tasks: Vec<Task<()>>,
    _feeds: Vec<Task<()>>,
    _observation: Subscription,
}

impl std::fmt::Debug for GovernancePill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernancePill")
            .field("open", &self.open)
            .finish_non_exhaustive()
    }
}

impl GovernancePill {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        let mut feeds_running = Vec::new();
        let mut sync = None;
        if let Some(engine) = feeds::sync_engine(&services) {
            sync = Some(engine.status());
            feeds_running.push(runtime::pump(
                cx,
                engine.subscribe(),
                Self::on_sync,
                |_, _, _| {},
            ));
        }
        if let Some(governance) = feeds::governance(&services) {
            governance.set_verdicts_enabled(
                services
                    .settings()
                    .read(|settings| settings.verdict_enabled),
            );
        }
        Self {
            app,
            services,
            sync,
            hovered: false,
            open: false,
            tasks: Vec::new(),
            _feeds: feeds_running,
            _observation: observation,
        }
    }

    fn on_sync(&mut self, status: SyncStatus, cx: &mut Context<Self>) {
        self.sync = Some(status);
        cx.notify();
    }

    fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            cx.notify();
        }
    }

    fn set_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.hovered != hovered {
            self.hovered = hovered;
            cx.notify();
        }
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.retain(|task| !task.is_ready());
        self.tasks.push(task);
    }

    pub(super) fn sync_now(&mut self, cx: &mut Context<Self>) {
        self.set_open(false, cx);
        actions::sync_now(&self.services, &self.app, cx);
    }

    pub(super) fn open_page(&mut self, cx: &mut Context<Self>) {
        self.set_open(false, cx);
        self.app.update(cx, |app, cx| {
            app.navigate(
                Route::Settings {
                    page: Some(PAGE_ID.to_owned()),
                },
                cx,
            );
        });
    }

    pub(super) fn switch_to(&mut self, id: AccountId, name: String, cx: &mut Context<Self>) {
        self.set_open(false, cx);
        let task = actions::activate(&self.services, self.app.clone(), (id, name), cx);
        self.keep(task);
    }

    pub(super) fn apply_fix(&mut self, fix: Fix, cx: &mut Context<Self>) {
        match fix {
            Fix::SyncNow | Fix::Wait => self.sync_now(cx),
            Fix::SignIn | Fix::ReviewKey | Fix::AskAdministrator => self.open_page(cx),
        }
    }

    fn model(&self, cx: &Context<Self>) -> PillModel {
        let fallback = self.services.identity().governance_label();
        pill(
            &self.app.read(cx).accounts,
            self.sync.as_ref(),
            Utc::now(),
            fallback,
        )
    }

    fn render_face(&self, model: &PillModel, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let tone = palette.tone(model.status.tone());
        let icon = Icon::new(model.status.icon())
            .size(size::ICON_SM)
            .color(tone.fg);
        let icon = if model.status.pulse() {
            motion::pulse("governance-pill-pulse", icon, cx)
        } else {
            icon.into_any_element()
        };
        let expanded = self.hovered || self.open || model.status.needs_attention();
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .child(icon)
            .child(
                div()
                    .max_w(px(LABEL_WIDTH))
                    .truncate()
                    .text_size(type_scale::SMALL)
                    .child(model.label.clone()),
            )
            .children(expanded.then(|| {
                div()
                    .text_size(type_scale::SMALL)
                    .text_color(if model.status.needs_attention() {
                        tone.fg
                    } else {
                        palette.text.muted
                    })
                    .whitespace_nowrap()
                    .child(text::pill_status(&model.status))
            }))
            .into_any_element()
    }
}

impl Render for GovernancePill {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model(cx);
        let face = self.render_face(&model, cx);
        let trigger = ToolkitButton::new("governance-pill-trigger")
            .ghost()
            .small()
            .accessibility_label(SharedString::from(model.label.clone()))
            .child(face);
        let view = cx.entity().downgrade();
        let mut popover = Popover::new("governance-pill-popover")
            .anchor(Anchor::TopRight)
            .open(self.open)
            .on_open_change(move |open, _, cx| {
                if view
                    .update(cx, |this, cx| this.set_open(*open, cx))
                    .is_err()
                {
                    tracing::trace!("governance pill closed before its popover");
                }
            })
            .trigger(trigger);
        if self.open {
            popover = popover.child(Self::render_menu(&model, cx));
        }
        div()
            .id("governance-pill")
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| this.set_hovered(*hovered, cx)))
            .child(popover)
    }
}
