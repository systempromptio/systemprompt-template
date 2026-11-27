//! The reads a state entity starts on tokio: one session, one project's
//! sessions, agents and commands, the project list. Store reads run on the
//! blocking pool.

use std::path::PathBuf;

use harness_config::AgentMode;
use harness_engine::{CommandInfo, Engine, EngineError};
use harness_store::{Project, Session, SessionId};

use crate::state::session_model::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentInfo {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectLoad {
    pub(crate) roots: Vec<Session>,
    pub(crate) children: Vec<Session>,
    pub(crate) agents: Vec<AgentInfo>,
    pub(crate) commands: Vec<CommandInfo>,
}

pub(crate) async fn load_session(engine: Engine, id: SessionId) -> Result<Snapshot, EngineError> {
    let session = engine.get(&id).await?;
    let messages = engine.messages(&id).await?;
    let store = engine.store().clone();
    let todo_id = id.clone();
    let todos = tokio::task::spawn_blocking(move || store.todos(&todo_id))
        .await
        .map_err(EngineError::Task)??;
    Ok(Snapshot {
        session,
        messages,
        todos,
        permissions: engine.pending_permissions(),
        questions: engine.pending_questions(),
        status: engine.status(&id),
    })
}

pub(crate) async fn load_project(
    engine: Engine,
    directory: PathBuf,
) -> Result<ProjectLoad, EngineError> {
    let roots = engine.list(&directory).await?;
    let agents = engine
        .agents(&directory)
        .await?
        .into_iter()
        .filter(|agent| agent.mode != AgentMode::Subagent)
        .map(|agent| AgentInfo {
            name: agent.name,
            description: agent.description,
        })
        .collect();
    let commands = engine.commands(&directory).await?;
    let store = engine.store().clone();
    let parents: Vec<SessionId> = roots.iter().map(|session| session.id.clone()).collect();
    let children = tokio::task::spawn_blocking(move || {
        let mut children = Vec::new();
        for parent in &parents {
            children.extend(store.child_sessions(parent)?);
        }
        Ok::<_, harness_store::StoreError>(children)
    })
    .await
    .map_err(EngineError::Task)??;
    Ok(ProjectLoad {
        roots,
        children,
        agents,
        commands,
    })
}

pub(crate) async fn load_projects(engine: Engine) -> Result<Vec<Project>, EngineError> {
    let store = engine.store().clone();
    let projects = tokio::task::spawn_blocking(move || store.list_projects())
        .await
        .map_err(EngineError::Task)??;
    Ok(projects)
}
