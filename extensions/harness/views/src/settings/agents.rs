//! The Agents and skills settings page: the project's agents and skills with
//! where each is defined, and a button that shows a file-defined one in the
//! file manager.

use std::path::{Path, PathBuf};

use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, FontWeight, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Task, Window, div,
};
use systemprompt_engine::config::{AgentDef, Origin, Skill};
use systemprompt_engine::run::EngineError;
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::{Badge, IconButton};
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};

use crate::app::{Services, runtime};
use crate::settings::{SettingsPage, layout, reveal};
use crate::state::AppState;
use crate::strings::settings as text;

pub(super) fn settings_page() -> SettingsPage {
    SettingsPage {
        id: "agents",
        title: text::AGENTS,
        icon: IconName::Sparkle,
        build,
    }
}

fn build(app: Entity<AppState>, services: Services, _window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| AgentsPage::new(app, services, cx)).into()
}

enum Content {
    NoProject,
    Loading,
    Failed,
    Ready {
        agents: Vec<AgentDef>,
        skills: Vec<Skill>,
    },
}

struct EntryRow {
    id: usize,
    title: String,
    badge: Option<&'static str>,
    description: Option<String>,
    location: String,
    folder: Option<PathBuf>,
}

struct AgentsPage {
    app: Entity<AppState>,
    services: Services,
    directory: Option<PathBuf>,
    content: Content,
    load_task: Option<Task<()>>,
    tasks: Vec<Task<()>>,
    _observation: Subscription,
}

impl AgentsPage {
    fn new(app: Entity<AppState>, services: Services, cx: &mut Context<Self>) -> Self {
        let observation = cx.observe(&app, |page: &mut Self, _, cx| page.follow_project(cx));
        let mut page = Self {
            app,
            services,
            directory: None,
            content: Content::NoProject,
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
        let job = async move {
            let agents = engine.agents(&directory).await?;
            let skills = engine.skills(&directory).await?;
            Ok::<_, EngineError>((agents, skills))
        };
        self.content = Content::Loading;
        let task = runtime::spawn(cx, self.services.runtime(), job, |page, result, cx| {
            page.content = match result {
                Ok((agents, skills)) => Content::Ready { agents, skills },
                Err(error) => {
                    tracing::warn!(error = %error, "agents and skills not loaded");
                    Content::Failed
                },
            };
            cx.notify();
        });
        self.load_task = Some(task);
        cx.notify();
    }

    fn show(&mut self, folder: PathBuf, cx: &Context<Self>) {
        let task = reveal::reveal(folder, &self.app, &self.services, cx);
        reveal::keep(&mut self.tasks, task);
    }

    fn render_entry(entry: EntryRow, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let title = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .child(
                div()
                    .text_size(type_scale::BODY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(SharedString::from(entry.title)),
            )
            .children(entry.badge.map(Badge::new));
        let button = entry.folder.map(|folder| {
            IconButton::new(
                ("show-in-folder", entry.id),
                IconName::Folder,
                text::SHOW_IN_FOLDER,
            )
            .small()
            .on_click(cx.listener(move |page, _, _, cx| page.show(folder.clone(), cx)))
        });
        div()
            .flex()
            .flex_row()
            .items_start()
            .gap(space::MD)
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(space::XXS)
                    .child(title)
                    .children(entry.description.map(|description| {
                        div()
                            .text_size(type_scale::SMALL)
                            .text_color(palette.text.secondary)
                            .child(SharedString::from(description))
                    }))
                    .child(layout::mono(entry.location, cx)),
            )
            .children(button)
            .into_any_element()
    }

    fn render_lists(agents: &[AgentDef], skills: &[Skill], cx: &Context<Self>) -> Vec<AnyElement> {
        let agent_rows: Vec<AnyElement> = if agents.is_empty() {
            vec![layout::note(text::NO_AGENTS, cx)]
        } else {
            agents
                .iter()
                .enumerate()
                .map(|(index, agent)| Self::render_entry(agent_entry(index, agent), cx))
                .collect()
        };
        let offset = agents.len();
        let skill_rows: Vec<AnyElement> = if skills.is_empty() {
            vec![layout::note(text::NO_SKILLS, cx)]
        } else {
            skills
                .iter()
                .enumerate()
                .map(|(index, skill)| Self::render_entry(skill_entry(offset + index, skill), cx))
                .collect()
        };
        vec![
            layout::card(text::AGENT_LIST, agent_rows),
            layout::card(text::SKILL_LIST, skill_rows),
        ]
    }
}

fn agent_entry(id: usize, agent: &AgentDef) -> EntryRow {
    let (location, folder) = match &agent.origin {
        Origin::Builtin => (text::ORIGIN_BUILTIN.to_owned(), None),
        Origin::Settings => (text::ORIGIN_SETTINGS.to_owned(), None),
        Origin::File(path) => (
            path.display().to_string(),
            path.parent().map(Path::to_path_buf),
        ),
    };
    EntryRow {
        id,
        title: agent.name.clone(),
        badge: Some(text::mode(agent.mode)),
        description: agent.description.clone(),
        location,
        folder,
    }
}

fn skill_entry(id: usize, skill: &Skill) -> EntryRow {
    EntryRow {
        id,
        title: skill.name.clone(),
        badge: None,
        description: Some(skill.description.clone()).filter(|description| !description.is_empty()),
        location: skill.location.display().to_string(),
        folder: Some(skill.dir.clone()),
    }
}

impl Render for AgentsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.content {
            Content::NoProject => vec![layout::empty(text::NO_PROJECT)],
            Content::Loading => vec![layout::loading()],
            Content::Failed => vec![layout::empty(text::LOAD_FAILED)],
            Content::Ready { agents, skills } => Self::render_lists(agents, skills, cx),
        };
        layout::page("settings-agents", text::AGENTS, text::AGENTS_BODY, body, cx)
    }
}
