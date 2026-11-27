//! The About settings page: which build this is, who is signed in, where the
//! app keeps its data and logs (each with an open button), and the licence
//! note.

use std::path::PathBuf;

use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, Render, SharedString,
    Subscription, Task, Window,
};
use harness_ui::IconName;
use harness_ui::components::Button;

use crate::app::Services;
use crate::state::AppState;
use crate::strings::settings as text;
use crate::views::settings::{SettingsPage, layout, reveal};

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "about",
        title: text::ABOUT,
        icon: IconName::Info,
        build,
    }
}

fn build(app: Entity<AppState>, services: Services, _window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        AboutPage {
            app,
            services,
            tasks: Vec::new(),
            _observation: observation,
        }
    })
    .into()
}

struct AboutPage {
    app: Entity<AppState>,
    services: Services,
    tasks: Vec<Task<()>>,
    _observation: Subscription,
}

impl AboutPage {
    fn open(&mut self, folder: PathBuf, cx: &Context<Self>) {
        let task = reveal::reveal(folder, &self.app, &self.services, cx);
        reveal::keep(&mut self.tasks, task);
    }

    fn folder_row(
        id: &'static str,
        title: &'static str,
        folder: PathBuf,
        cx: &Context<Self>,
    ) -> AnyElement {
        let shown = SharedString::from(folder.display().to_string());
        let button = Button::new(id)
            .label(text::OPEN_FOLDER)
            .icon(IconName::Folder)
            .small()
            .on_click(cx.listener(move |page, _, _, cx| page.open(folder.clone(), cx)));
        layout::row(title, Some(shown), button, cx)
    }

    fn signed_in_as(&self, cx: &App) -> SharedString {
        let state = self.app.read(cx);
        state
            .accounts
            .active_account()
            .filter(|_| !state.needs_sign_in())
            .map_or_else(
                || SharedString::new_static(text::NOT_SIGNED_IN),
                |account| {
                    SharedString::from(
                        account
                            .user
                            .clone()
                            .unwrap_or_else(|| account.label.clone()),
                    )
                },
            )
    }
}

impl Render for AboutPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let identity = self.services.identity();
        let paths = self.services.paths();
        let build = vec![
            layout::row(
                text::APP_NAME,
                None,
                layout::note(SharedString::new_static(identity.display_name()), cx),
                cx,
            ),
            layout::row(
                text::VERSION,
                None,
                layout::note(SharedString::new_static(identity.version()), cx),
                cx,
            ),
            layout::row(
                text::CHANNEL,
                None,
                layout::note(
                    SharedString::new_static(text::channel(identity.channel())),
                    cx,
                ),
                cx,
            ),
            layout::row(
                text::SIGNED_IN_AS,
                None,
                layout::note(self.signed_in_as(cx), cx),
                cx,
            ),
        ];
        let folders = vec![
            Self::folder_row(
                "about-data-folder",
                text::DATA_FOLDER,
                paths.root.clone(),
                cx,
            ),
            Self::folder_row("about-log-folder", text::LOG_FOLDER, paths.logs.clone(), cx),
        ];
        let body = vec![
            layout::card(text::ABOUT, build),
            layout::card(text::DATA_FOLDER, folders),
            layout::card(text::LICENCES, vec![layout::note(text::LICENCES_BODY, cx)]),
        ];
        layout::page("settings-about", text::ABOUT, text::ABOUT_BODY, body, cx)
    }
}
