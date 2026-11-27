//! The General settings page: the colour scheme (applied live), the project
//! opened at start, the sidebar, and the composer's preferences. Every change
//! is written through `SettingsStore::update`.

use std::path::PathBuf;

use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, AsyncApp, Context, Entity, IntoElement,
    ParentElement as _, PathPromptOptions, Render, SharedString, Styled as _, Subscription, Task,
    WeakEntity, Window, div, px,
};
use systemprompt_harness_shared::components::{Button, Select, SelectOption, Toggle};
use systemprompt_harness_shared::theme::space;
use systemprompt_harness_shared::{IconName, Scheme};

use crate::app::settings::{DeliveryPref, UiSettings};
use crate::app::{Services, runtime};
use crate::settings::{SettingsPage, layout, reveal};
use crate::state::AppState;
use crate::strings::settings as text;

const SELECT_WIDTH: f32 = 200.0;
const DELIVERIES: [DeliveryPref; 2] = [DeliveryPref::Steer, DeliveryPref::Queue];

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "general",
        title: text::GENERAL,
        icon: IconName::Settings,
        build,
    }
}

fn build(app: Entity<AppState>, services: Services, _window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| GeneralPage::new(app, services, cx)).into()
}

const fn delivery_key(delivery: DeliveryPref) -> &'static str {
    match delivery {
        DeliveryPref::Steer => "steer",
        DeliveryPref::Queue => "queue",
    }
}

struct GeneralPage {
    app: Entity<AppState>,
    services: Services,
    tasks: Vec<Task<()>>,
    _settings: Task<()>,
    _observation: Subscription,
}

impl GeneralPage {
    fn new(app: Entity<AppState>, services: Services, cx: &mut Context<Self>) -> Self {
        let settings = runtime::pump(
            cx,
            services.settings().subscribe(),
            |_, _, cx| cx.notify(),
            |_, _, cx| cx.notify(),
        );
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            services,
            tasks: Vec::new(),
            _settings: settings,
            _observation: observation,
        }
    }

    fn choose_start_folder(&mut self, cx: &Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(SharedString::new_static(text::CHOOSE_FOLDER_PROMPT)),
        });
        let task = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let folder = match chosen.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) | Err(_) => None,
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "folder dialog failed");
                    None
                },
            };
            let Some(folder) = folder else { return };
            if this
                .update(cx, |this, _| this.set_start_folder(Some(folder)))
                .is_err()
            {
                tracing::trace!("settings page closed before the folder was chosen");
            }
        });
        reveal::keep(&mut self.tasks, task);
    }

    fn set_start_folder(&self, folder: Option<PathBuf>) {
        self.services
            .settings()
            .update(|settings| settings.last_project = folder);
    }

    fn appearance(&self, settings: &UiSettings, cx: &App) -> AnyElement {
        let store = self.services.settings().clone();
        let options =
            Scheme::ALL.map(|scheme| SelectOption::new(scheme.key(), text::scheme(scheme)));
        let select = Select::new("settings-scheme", options)
            .selected(Some(SharedString::new_static(settings.scheme().key())))
            .width(px(SELECT_WIDTH))
            .on_select(move |value, _, cx| {
                let Some(scheme) = Scheme::from_key(value) else {
                    return;
                };
                store.update(|settings| settings.set_scheme(scheme));
                systemprompt_harness_shared::theme::set_scheme(cx, scheme);
            });
        let row = layout::row(
            text::SCHEME,
            Some(SharedString::new_static(text::SCHEME_BODY)),
            select,
            cx,
        );
        layout::card(text::APPEARANCE, vec![row])
    }

    fn startup(&self, settings: &UiSettings, cx: &Context<Self>) -> AnyElement {
        let folder = settings
            .last_project
            .clone()
            .unwrap_or_else(|| self.services.paths().home.clone());
        let buttons = div()
            .flex()
            .flex_row()
            .gap(space::SM)
            .child(
                Button::new("settings-start-folder")
                    .label(text::CHOOSE_FOLDER)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.choose_start_folder(cx))),
            )
            .child(
                Button::new("settings-start-home")
                    .label(text::USE_HOME)
                    .small()
                    .ghost()
                    .disabled(settings.last_project.is_none())
                    .on_click(cx.listener(|this, _, _, _| this.set_start_folder(None))),
            );
        let folder_row = layout::row(
            text::START_FOLDER,
            Some(SharedString::from(folder.display().to_string())),
            buttons,
            cx,
        );
        let app = self.app.clone();
        let sidebar = Toggle::new("settings-sidebar")
            .on(self.app.read(cx).sidebar_open)
            .on_change(move |on, _, cx| {
                app.update(cx, |app, cx| {
                    if app.sidebar_open != on {
                        app.toggle_sidebar(cx);
                    }
                });
            });
        let sidebar_row = layout::row(
            text::SHOW_SIDEBAR,
            Some(SharedString::new_static(text::SHOW_SIDEBAR_BODY)),
            sidebar,
            cx,
        );
        layout::card(
            text::STARTUP,
            vec![
                folder_row,
                layout::note(text::START_FOLDER_BODY, cx),
                sidebar_row,
            ],
        )
    }

    fn composer(&self, settings: &UiSettings, cx: &App) -> AnyElement {
        let store = self.services.settings().clone();
        let enter = Toggle::new("settings-enter-sends")
            .on(settings.composer.enter_sends)
            .on_change(move |on, _, _| {
                store.update(|settings| settings.composer.enter_sends = on);
            });
        let store = self.services.settings().clone();
        let options = DELIVERIES
            .map(|delivery| SelectOption::new(delivery_key(delivery), text::delivery(delivery)));
        let delivery = Select::new("settings-delivery", options)
            .selected(Some(SharedString::new_static(delivery_key(
                settings.composer.delivery,
            ))))
            .width(px(SELECT_WIDTH))
            .on_select(move |value, _, _| {
                let Some(choice) = DELIVERIES
                    .into_iter()
                    .find(|delivery| delivery_key(*delivery) == value.as_ref())
                else {
                    return;
                };
                store.update(|settings| settings.composer.delivery = choice);
            });
        layout::card(
            text::COMPOSER,
            vec![
                layout::row(
                    text::ENTER_SENDS,
                    Some(SharedString::new_static(text::ENTER_SENDS_BODY)),
                    enter,
                    cx,
                ),
                layout::row(
                    text::DELIVERY,
                    Some(SharedString::new_static(text::DELIVERY_BODY)),
                    delivery,
                    cx,
                ),
            ],
        )
    }
}

impl Render for GeneralPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.services.settings().get();
        let body = vec![
            self.appearance(&settings, cx),
            self.startup(&settings, cx),
            self.composer(&settings, cx),
        ];
        layout::page(
            "settings-general",
            text::GENERAL,
            text::GENERAL_BODY,
            body,
            cx,
        )
    }
}
