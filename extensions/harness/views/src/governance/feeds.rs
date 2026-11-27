//! The sync and governance handles the shell may have registered, and what the
//! applied instance holds.
//!
//! That is its managed skills as the engine loaded them, its managed MCP
//! servers with their live status, and its rules count.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use systemprompt_engine::config::Skill;
use systemprompt_engine::mcp::{ServerStatus, StatusMap};
use systemprompt_engine::run::Engine;
use systemprompt_harness::governance::Governance;
use systemprompt_harness::sync::SyncEngine;

use crate::app::Services;

pub(crate) fn sync_engine(services: &Services) -> Option<Arc<SyncEngine>> {
    services.handles().get::<SyncEngine>()
}

pub(crate) fn governance(services: &Services) -> Option<Arc<Governance>> {
    services.handles().get::<Governance>()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppliedView {
    pub instance_url: String,
    pub mcp: Vec<String>,
    pub rules: u32,
    pub governed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpState {
    Connected { tools: usize },
    Connecting,
    Failed,
    Disabled,
    NeedsAuth,
    NotStarted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldSkill {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldServer {
    pub(crate) name: String,
    pub state: McpState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Held {
    pub(crate) instance_url: String,
    pub skills: Vec<HeldSkill>,
    pub servers: Vec<HeldServer>,
    pub rules: u32,
    pub(crate) governed: bool,
}

pub fn held(
    applied: AppliedView,
    skills: &[Skill],
    managed_root: &Path,
    statuses: &StatusMap,
) -> Held {
    let mut held_skills: Vec<HeldSkill> = skills
        .iter()
        .filter(|skill| skill.dir.starts_with(managed_root))
        .map(|skill| HeldSkill {
            name: skill.name.clone(),
            description: Some(skill.description.trim().to_owned())
                .filter(|description| !description.is_empty()),
        })
        .collect();
    held_skills.sort_by(|left, right| left.name.cmp(&right.name));
    let servers = applied
        .mcp
        .into_iter()
        .map(|name| {
            let state = statuses.get(&name).map_or(McpState::NotStarted, mcp_state);
            HeldServer { name, state }
        })
        .collect();
    Held {
        instance_url: applied.instance_url,
        skills: held_skills,
        servers,
        rules: applied.rules,
        governed: applied.governed,
    }
}

const fn mcp_state(status: &ServerStatus) -> McpState {
    match status {
        ServerStatus::Connected { tools } => McpState::Connected { tools: *tools },
        ServerStatus::Connecting => McpState::Connecting,
        ServerStatus::Failed { .. } => McpState::Failed,
        ServerStatus::Disabled => McpState::Disabled,
        ServerStatus::NeedsAuth => McpState::NeedsAuth,
    }
}

pub(crate) async fn load_held(
    governance: Arc<Governance>,
    engine: Engine,
    directory: PathBuf,
    managed_root: PathBuf,
) -> Option<Held> {
    let applied = governance.applied().map(|applied| AppliedView {
        instance_url: applied.instance_url,
        mcp: applied.mcp.into_iter().map(|server| server.name).collect(),
        rules: applied.counts.rules,
        governed: applied.governance_plugin.is_some(),
    })?;
    let skills = match engine.skills(&directory).await {
        Ok(skills) => skills,
        Err(error) => {
            tracing::warn!(error = %error, "managed skills not listed");
            Vec::new()
        },
    };
    let statuses = match engine.mcp_status(&directory).await {
        Ok(statuses) => statuses,
        Err(error) => {
            tracing::warn!(error = %error, "managed MCP status not read");
            StatusMap::new()
        },
    };
    Some(held(applied, &skills, &managed_root, &statuses))
}
