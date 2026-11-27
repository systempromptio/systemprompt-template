//! The GPUI side of a run: the main window over the app state, later starts
//! and opened links brought to it, the services stopped when the app quits,
//! and the window that reports a startup failure.
//!
//! A link the platform hands the running app (macOS delivers a URL of the
//! app's scheme through the toolkit's open-URLs callback rather than the
//! command line) is forwarded into the same request channel as a start handed
//! over by a second instance; the callback is registered before the run.

use std::future::Future;
use std::path::PathBuf;

use gpui_kit::component::TitleBar;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, AsyncApp, Entity, WindowBounds, WindowOptions, px, size,
};
use systemprompt_traits::OwnedTask;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use systemprompt_harness_views::app::Services;
use systemprompt_harness_views::app::deeplink::LaunchRequest;
use systemprompt_harness_views::fatal::FatalView;
use systemprompt_harness_views::frame::Frame;
use systemprompt_harness_views::state::AppState;
use systemprompt_harness_views::{keymap, window_state};

const FATAL_WIDTH: f32 = 520.0;
const FATAL_HEIGHT: f32 = 300.0;
const SHUTDOWN_TASK: &str = "services-shutdown";

pub(crate) struct Launch {
    pub(crate) services: Services,
    pub(crate) directory: PathBuf,
    pub(crate) links: Vec<String>,
    pub(crate) forward: UnboundedSender<LaunchRequest>,
    pub(crate) requests: UnboundedReceiver<LaunchRequest>,
}

pub(crate) fn run(launch: Launch) {
    let Launch {
        services,
        directory,
        links,
        forward,
        requests,
    } = launch;
    let application = gpui_kit::application().with_assets(systemprompt_harness_shared::Assets);
    application.on_open_urls(move |links| forward_links(&forward, links));
    application.run(move |cx| {
        let identity = services.identity();
        cx.set_app_identity(identity.app_id(), identity.display_name());
        systemprompt_harness_shared::init(cx);
        systemprompt_harness_shared::theme::set_scheme(cx, services.settings().get().scheme());
        keymap::bind(cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let on_quit = services.clone();
        cx.on_app_quit(move |_| quit(on_quit.clone())).detach();

        let app = cx.new(|cx| AppState::new(services.clone(), cx));
        app.update(cx, |app, cx| {
            app.open_project(directory, cx);
            if !links.is_empty() {
                app.handle_request(
                    &LaunchRequest {
                        directory: None,
                        links,
                    },
                    cx,
                );
            }
        });
        match open_main_window(&app, &services, cx) {
            Ok(handle) => {
                follow_requests(app, handle, requests, cx);
                cx.activate(true);
            },
            Err(error) => {
                tracing::error!(error = %error, "main window did not open");
                cx.quit();
            },
        }
    });
}

fn open_main_window(
    app: &Entity<AppState>,
    services: &Services,
    cx: &mut App,
) -> anyhow::Result<AnyWindowHandle> {
    let identity = services.identity();
    let saved = services.settings().get().window;
    let options = WindowOptions {
        window_bounds: Some(window_state::initial_bounds(saved, cx)),
        window_min_size: Some(window_state::min_size()),
        app_id: Some(identity.app_id().to_owned()),
        ..TitleBar::window_options()
    };
    let (app, services) = (app.clone(), services.clone());
    let (handle, _frame) = gpui_kit::open_window(options, cx, move |window, cx| {
        window.set_window_title(identity.display_name());
        cx.new(|cx| Frame::new(app, services, window, cx))
    })?;
    Ok(handle)
}

fn follow_requests(
    app: Entity<AppState>,
    handle: AnyWindowHandle,
    mut requests: UnboundedReceiver<LaunchRequest>,
    cx: &App,
) {
    cx.spawn(async move |cx: &mut AsyncApp| {
        while let Some(request) = requests.recv().await {
            app.update(cx, |app, cx| app.handle_request(&request, cx));
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_err()
            {
                break;
            }
        }
    })
    .detach();
}

fn forward_links(forward: &UnboundedSender<LaunchRequest>, links: Vec<String>) {
    let request = LaunchRequest {
        directory: None,
        links,
    };
    if forward.send(request).is_err() {
        tracing::debug!("a link arrived after the window closed");
    }
}

fn quit(services: Services) -> impl Future<Output = ()> {
    let runtime = services.runtime().clone();
    let job = {
        let _entered = runtime.enter();
        OwnedTask::spawn(SHUTDOWN_TASK, async move { services.shutdown().await })
    };
    async move {
        if let Err(error) = job.join_named().await {
            tracing::warn!(error = %error, "shutdown task failed");
        }
    }
}

pub(crate) fn show_fatal() {
    gpui_kit::application()
        .with_assets(systemprompt_harness_shared::Assets)
        .run(|cx| {
            systemprompt_harness_shared::init(cx);
            cx.on_window_closed(|cx, _| cx.quit()).detach();
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(
                    size(px(FATAL_WIDTH), px(FATAL_HEIGHT)),
                    cx,
                )),
                ..WindowOptions::default()
            };
            let opened = gpui_kit::open_window(options, cx, |window, cx| {
                window.set_window_title(systemprompt_harness_brand::identity().display_name());
                cx.new(|_| FatalView)
            });
            match opened {
                Ok(_) => cx.activate(true),
                Err(error) => {
                    tracing::error!(error = %error, "the startup failure window did not open");
                    cx.quit();
                },
            }
        });
}
