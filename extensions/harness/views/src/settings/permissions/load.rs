//! Gathering the effective rules for one project and agent: the agent's
//! built-in defaults, the user's settings, each project settings file from the
//! project root down, and the agent's own rules, in the order they are checked.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::path::{Path, PathBuf};

use harness_config::{AgentDef, AgentMode, Layout, Paths, Ruleset, Worktree};
use harness_engine::{Engine, EngineError};

use crate::views::settings::permissions::rules::{
    self, FileRules, Group, RulesError, ShownRule, Source,
};

#[derive(Debug)]
pub(super) struct Request {
    pub(super) engine: Engine,
    pub(super) paths: Paths,
    pub(super) layout: Layout,
    pub(super) directory: PathBuf,
    pub(super) agent: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ProblemKind {
    #[error("the file could not be read")]
    Read(#[source] std::io::Error),
    #[error("the file's rules could not be read")]
    Rules(#[source] RulesError),
}

#[derive(Debug)]
pub(super) struct Problem {
    pub(super) path: PathBuf,
    pub(super) kind: ProblemKind,
}

#[derive(Debug, Default)]
pub(super) struct RulesView {
    pub(super) agent: Option<String>,
    pub(super) user: FileRules,
    pub(super) sources: Vec<Source>,
    pub(super) groups: Vec<Group>,
    pub(super) problems: Vec<Problem>,
}

pub(super) async fn load(request: Request) -> Result<RulesView, EngineError> {
    let Request {
        engine,
        paths,
        layout,
        directory,
        agent,
    } = request;
    let agents = engine.agents(&directory).await?;
    let chosen = choose_agent(agents, agent.as_deref());
    tokio::task::spawn_blocking(move || assemble(&paths, &layout, &directory, chosen.as_ref()))
        .await
        .map_err(EngineError::Task)
}

fn choose_agent(agents: Vec<AgentDef>, wanted: Option<&str>) -> Option<AgentDef> {
    let primary = |agent: &AgentDef| agent.mode != AgentMode::Subagent;
    let mut agents = agents.into_iter().filter(primary);
    match wanted {
        Some(name) => agents.find(|agent| agent.name == name),
        None => agents.next(),
    }
}

fn assemble(
    paths: &Paths,
    layout: &Layout,
    directory: &Path,
    agent: Option<&AgentDef>,
) -> RulesView {
    let mut view = RulesView {
        agent: agent.map(|agent| agent.name.clone()),
        ..RulesView::default()
    };
    let mut shown = Vec::new();
    if let Some(agent) = agent {
        push_ruleset(
            &mut view,
            &mut shown,
            Source::Defaults(agent.name.clone()),
            &agent.defaults,
        );
    }
    let user_file = layout.global_settings(paths);
    let user = read_rules(&user_file, &paths.home, &mut view.problems).unwrap_or_default();
    let user_index = view.sources.len();
    view.sources.push(Source::User(user_file));
    shown.extend(user.entries.iter().map(|entry| ShownRule {
        rule: entry.rule.clone(),
        source: user_index,
        removal: rules::remove_edit(entry),
    }));
    view.user = user;
    for file in project_files(layout, directory) {
        let Some(project) = read_rules(&file, &paths.home, &mut view.problems) else {
            continue;
        };
        let index = view.sources.len();
        view.sources.push(Source::Project(file));
        shown.extend(project.entries.into_iter().map(|entry| ShownRule {
            rule: entry.rule,
            source: index,
            removal: None,
        }));
    }
    if let Some(agent) = agent {
        push_ruleset(
            &mut view,
            &mut shown,
            Source::Agent(agent.name.clone()),
            &agent.permission,
        );
    }
    view.groups = rules::group(shown);
    view
}

fn push_ruleset(view: &mut RulesView, shown: &mut Vec<ShownRule>, source: Source, rules: &Ruleset) {
    if rules.is_empty() {
        return;
    }
    let index = view.sources.len();
    view.sources.push(source);
    shown.extend(rules.rules().iter().map(|rule| ShownRule {
        rule: rule.clone(),
        source: index,
        removal: None,
    }));
}

fn project_files(layout: &Layout, directory: &Path) -> Vec<PathBuf> {
    let worktree = Worktree::detect(directory);
    worktree
        .chain(directory)
        .iter()
        .flat_map(|dir| layout.project_settings(dir))
        .filter(|file| file.is_file())
        .collect()
}

fn read_rules(path: &Path, home: &Path, problems: &mut Vec<Problem>) -> Option<FileRules> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Some(FileRules {
                entries: Vec::new(),
                editable: true,
            });
        },
        Err(error) => {
            tracing::warn!(path = %path.display(), error = %error, "settings file unreadable");
            problems.push(Problem {
                path: path.to_path_buf(),
                kind: ProblemKind::Read(error),
            });
            return None;
        },
    };
    match rules::parse_file(&text, home) {
        Ok(rules) => Some(rules),
        Err(error) => {
            tracing::warn!(path = %path.display(), error = %error, "settings rules unreadable");
            problems.push(Problem {
                path: path.to_path_buf(),
                kind: ProblemKind::Rules(error),
            });
            None
        },
    }
}
