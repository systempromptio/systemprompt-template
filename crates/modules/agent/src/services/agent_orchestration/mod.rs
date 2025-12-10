//! Agent Orchestration - Clean Database-Driven Implementation
//!
//! This module provides a complete replacement for the old agent management
//! system. Key principles:
//! - Database is the only source of truth (no in-memory state)
//! - Processes are fully detached (survive orchestrator restarts)
//! - Direct PID management (validate against OS, not memory)

pub mod database;
pub mod lifecycle;
pub mod monitor;
pub mod orchestrator;
pub mod port_manager;
pub mod process;
pub mod reconciler;

pub use orchestrator::AgentOrchestrator;
pub use port_manager::PortManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Running {
        pid: u32,
        port: u16,
    },
    Failed {
        reason: String,
        last_attempt: Option<String>,
        retry_count: u32,
    },
}

#[derive(Debug, Clone)]
pub struct AgentRuntimeConfig {
    pub id: String,
    pub name: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<String>,
}

impl ValidationReport {
    pub const fn new() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
        }
    }

    pub fn with_issue(issue: String) -> Self {
        Self {
            valid: false,
            issues: vec![issue],
        }
    }

    pub fn add_issue(&mut self, issue: String) {
        self.valid = false;
        self.issues.push(issue);
    }
}

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("Agent {0} not found")]
    AgentNotFound(String),

    #[error("Agent {0} already running")]
    AgentAlreadyRunning(String),

    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Health check timeout for agent {0}")]
    HealthCheckTimeout(String),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

pub type OrchestrationResult<T> = Result<T, OrchestrationError>;
