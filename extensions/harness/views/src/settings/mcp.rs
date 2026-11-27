//! The MCP servers settings page: each server the open project runs with its
//! connection status, refreshed on demand. Servers the instance manages are
//! marked as managed and offer no actions; the user's own servers are
//! configured in the settings folder this page opens.

use std::path::PathBuf;

use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, FontWeight, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Task, Window, div,
};
use harness_engine::EngineError;
use harness_mcp::{ServerStatus, StatusMap};
use harness_ui::components::{Badge, Button, IconButton, StatusDot};
use harness_ui::theme::{space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};

use crate::app::{Services, runtime};
use crate::state::AppState;
use crate::strings::settings as text;
use crate::views::settings::{SettingsPage, layout, reveal};

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "mcp",
        title: text::MCP,
        icon: IconName::Plug,
        build,
    }
}

fn build(app: Entity<AppState>, services: Services, _window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| McpPage::new(app, services, cx)).into()
}

enum Content {
    NoProject,
    Loading,
    Failed,
    Ready {
        servers: StatusMap,
        managed: Vec<String>,
    },
}

struct McpPage {
    app: Entity<AppState>,
    services: Services,
    directory: Option<PathBuf>,
    content: Content,
    refreshing: bool,
    load_task: Option<Task<()>>,
    tasks: Vec<Task<()>>,
    _observation: Subscription,
}

fn status_view(status: &ServerStatus) -> (Tone, SharedString, bool) {
    match status {
        ServerStatus::Connecting => (
            Tone::Info,
            SharedString::new_static(text::STATUS_CONNECTING),
            true,
        ),
        ServerStatus::Connected { tools } => (
            Tone::Success,
            SharedString::from(text::connected(*tools)),
            false,
        ),
        ServerStatus::Failed { error } => (
            Tone::Danger,
            SharedString::from(text::with_detail(text::STATUS_FAILED, error)),
            false,
        ),
        ServerStatus::Disabled => (
            Tone::Neutral,
            SharedString::new_static(text::STATUS_DISABLED),
            false,
        ),
        ServerStatus::NeedsAuth => (
            Tone::Warning,
            SharedString::new_static(text::STATUS_NEEDS_AUTH),
            false,
        ),
    }
}

impl McpPage {
    fn new(app: Entity<AppState>, services: Services, cx: &mut Context<Self>) -> Self {
        let observation = cx.observe(&app, |page: &mut Self, _, cx| page.follow_project(cx));
        let mut page = Self {
            app,
            services,
            directory: None,
            content: Content::NoProject,
            refreshing: false,
            load_task: None,
            tasks: Vec::new(),
            _observation: observation,
        };
        page.directory = page.project(cx);
        page.reload(cx);
        page
    }

    fn project(&self, cx: &App) -> Option<PathBuf> {
        self.app
            .read(cx)
            .project
            .as_ref()
            .map(|project| project.directory.clone())
    }

    fn follow_project(&mut self, cx: &mut Context<Self>) {
        let directory = self.project(cx);
        if directory != self.directory {
            self.directory = directory;
            self.content = Content::Loading;
            self.reload(cx);
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(directory) = self.directory.clone() else {
            self.content = Content::NoProject;
            cx.notify();
            return;
        };
        let engine = self.services.engine().clone();
        let governor = self.services.governor();
        let job = async move {
            let servers = engine.mcp_status(&directory).await?;
            let managed: Vec<String> = governor
                .mcp_servers()
                .into_iter()
                .map(|server| server.name)
                .collect();
            Ok::<_, EngineError>((servers, managed))
        };
        if !matches!(self.content, Content::Ready { .. }) {
            self.content = Content::Loading;
        }
        self.refreshing = true;
        let task = runtime::spawn(cx, self.services.runtime(), job, |page, result, cx| {
            page.refreshing = false;
            page.content = match result {
                Ok((servers, managed)) => Content::Ready { servers, managed },
                Err(error) => {
                    tracing::warn!(error = %error, "MCP status not loaded");
                    Content::Failed
                },
            };
            cx.notify();
        });
        self.load_task = Some(task);
        cx.notify();
    }

    fn open_settings_folder(&mut self, cx: &Context<Self>) {
        let folder = self.services.paths().config.clone();
        let task = reveal::reveal(folder, &self.app, &self.services, cx);
        reveal::keep(&mut self.tasks, task);
    }

    fn render_actions(&self, cx: &Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .gap(space::SM)
            .child(
                IconButton::new("mcp-refresh", IconName::Sync, text::REFRESH)
                    .small()
                    .loading(self.refreshing)
                    .disabled(self.directory.is_none())
                    .on_click(cx.listener(|page, _, _, cx| page.reload(cx))),
            )
            .child(
                Button::new("mcp-settings-folder")
                    .label(text::OPEN_SETTINGS_FOLDER)
                    .icon(IconName::Folder)
                    .small()
                    .on_click(cx.listener(|page, _, _, cx| page.open_settings_folder(cx))),
            )
            .into_any_element()
    }

    fn render_server(
        index: usize,
        name: &str,
        status: &ServerStatus,
        managed: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let (tone, label, live) = status_view(status);
        let badges = managed.then(|| {
            div()
                .flex()
                .flex_row()
                .gap(space::XS)
                .child(
                    Badge::new(text::MANAGED)
                        .tone(Tone::Accent)
                        .icon(IconName::Shield),
                )
                .child(Badge::new(text::READ_ONLY))
        });
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .w_full()
            .child(StatusDot::new(("mcp-status", index), tone).pulse(live))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(type_scale::BODY)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text.primary)
                            .child(SharedString::from(name.to_owned())),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(type_scale::SMALL)
                            .text_color(palette.text.muted)
                            .child(label),
                    ),
            )
            .children(badges)
            .into_any_element()
    }

    fn render_servers(
        servers: &StatusMap,
        managed: &[String],
        cx: &Context<Self>,
    ) -> Vec<AnyElement> {
        let rows: Vec<AnyElement> = if servers.is_empty() {
            vec![layout::note(text::NO_SERVERS, cx)]
        } else {
            servers
                .iter()
                .enumerate()
                .map(|(index, (name, status))| {
                    Self::render_server(
                        index,
                        name,
                        status,
                        managed.iter().any(|known| known == name),
                        cx,
                    )
                })
                .collect()
        };
        vec![layout::card(text::MCP, rows)]
    }
}

impl Render for McpPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut body = vec![self.render_actions(cx)];
        match &self.content {
            Content::NoProject => body.push(layout::empty(text::NO_PROJECT)),
            Content::Loading => body.push(layout::loading()),
            Content::Failed => body.push(layout::empty(text::LOAD_FAILED)),
            Content::Ready { servers, managed } => {
                body.extend(Self::render_servers(servers, managed, cx));
            },
        }
        layout::page("settings-mcp", text::MCP, text::MCP_BODY, body, cx)
    }
}
