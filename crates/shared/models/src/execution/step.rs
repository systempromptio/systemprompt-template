use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{SkillId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub String);

impl StepId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for StepId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for StepStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Invalid step status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    #[default]
    Understanding,
    Planning,
    SkillUsage,
    ToolExecution,
    Completion,
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Understanding => write!(f, "understanding"),
            Self::Planning => write!(f, "planning"),
            Self::SkillUsage => write!(f, "skill_usage"),
            Self::ToolExecution => write!(f, "tool_execution"),
            Self::Completion => write!(f, "completion"),
        }
    }
}

impl std::str::FromStr for StepType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "understanding" => Ok(Self::Understanding),
            "planning" => Ok(Self::Planning),
            "skill_usage" => Ok(Self::SkillUsage),
            "tool_execution" | "toolexecution" => Ok(Self::ToolExecution),
            "completion" => Ok(Self::Completion),
            _ => Err(format!("Invalid step type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedTool {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepContent {
    Understanding,
    Planning {
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        planned_tools: Option<Vec<PlannedTool>>,
    },
    SkillUsage {
        skill_id: SkillId,
        skill_name: String,
    },
    ToolExecution {
        tool_name: String,
        tool_arguments: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_result: Option<serde_json::Value>,
    },
    Completion,
}

impl StepContent {
    /// Create an Understanding step
    pub const fn understanding() -> Self {
        Self::Understanding
    }

    /// Create a Planning step with optional reasoning and planned tools
    pub const fn planning(reasoning: Option<String>, planned_tools: Option<Vec<PlannedTool>>) -> Self {
        Self::Planning {
            reasoning,
            planned_tools,
        }
    }

    /// Create a SkillUsage step
    pub fn skill_usage(skill_id: SkillId, skill_name: impl Into<String>) -> Self {
        Self::SkillUsage {
            skill_id,
            skill_name: skill_name.into(),
        }
    }

    /// Create a ToolExecution step (in-progress, no result yet)
    pub fn tool_execution(tool_name: impl Into<String>, tool_arguments: serde_json::Value) -> Self {
        Self::ToolExecution {
            tool_name: tool_name.into(),
            tool_arguments,
            tool_result: None,
        }
    }

    /// Create a Completion step
    pub const fn completion() -> Self {
        Self::Completion
    }

    pub const fn step_type(&self) -> StepType {
        match self {
            Self::Understanding => StepType::Understanding,
            Self::Planning { .. } => StepType::Planning,
            Self::SkillUsage { .. } => StepType::SkillUsage,
            Self::ToolExecution { .. } => StepType::ToolExecution,
            Self::Completion => StepType::Completion,
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Understanding => "Analyzing request...".to_string(),
            Self::Planning { .. } => "Planning response...".to_string(),
            Self::SkillUsage { skill_name, .. } => format!("Using {} skill...", skill_name),
            Self::ToolExecution { tool_name, .. } => format!("Running {}...", tool_name),
            Self::Completion => "Complete".to_string(),
        }
    }

    pub const fn is_instant(&self) -> bool {
        !matches!(self, Self::ToolExecution { .. })
    }

    pub fn tool_name(&self) -> Option<&str> {
        match self {
            Self::ToolExecution { tool_name, .. } => Some(tool_name),
            Self::SkillUsage { skill_name, .. } => Some(skill_name),
            _ => None,
        }
    }

    pub const fn tool_arguments(&self) -> Option<&serde_json::Value> {
        match self {
            Self::ToolExecution { tool_arguments, .. } => Some(tool_arguments),
            _ => None,
        }
    }

    pub const fn tool_result(&self) -> Option<&serde_json::Value> {
        match self {
            Self::ToolExecution { tool_result, .. } => tool_result.as_ref(),
            _ => None,
        }
    }

    pub fn reasoning(&self) -> Option<&str> {
        match self {
            Self::Planning { reasoning, .. } => reasoning.as_deref(),
            _ => None,
        }
    }

    pub fn planned_tools(&self) -> Option<&[PlannedTool]> {
        match self {
            Self::Planning { planned_tools, .. } => planned_tools.as_deref(),
            _ => None,
        }
    }

    pub fn with_tool_result(self, result: serde_json::Value) -> Self {
        match self {
            Self::ToolExecution {
                tool_name,
                tool_arguments,
                ..
            } => Self::ToolExecution {
                tool_name,
                tool_arguments,
                tool_result: Some(result),
            },
            other => other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStep {
    pub step_id: StepId,
    pub task_id: String,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub content: StepContent,
}

impl ExecutionStep {
    pub fn new(task_id: TaskId, content: StepContent) -> Self {
        let status = if content.is_instant() {
            StepStatus::Completed
        } else {
            StepStatus::InProgress
        };
        let now = Utc::now();
        let (completed_at, duration_ms) = if content.is_instant() {
            (Some(now), Some(0))
        } else {
            (None, None)
        };

        Self {
            step_id: StepId::new(),
            task_id: task_id.to_string(),
            status,
            started_at: now,
            completed_at,
            duration_ms,
            error_message: None,
            content,
        }
    }

    /// Create an understanding step
    pub fn understanding(task_id: TaskId) -> Self {
        Self::new(task_id, StepContent::understanding())
    }

    /// Create a planning step
    pub fn planning(
        task_id: TaskId,
        reasoning: Option<String>,
        planned_tools: Option<Vec<PlannedTool>>,
    ) -> Self {
        Self::new(task_id, StepContent::planning(reasoning, planned_tools))
    }

    /// Create a skill usage step
    pub fn skill_usage(task_id: TaskId, skill_id: SkillId, skill_name: impl Into<String>) -> Self {
        Self::new(task_id, StepContent::skill_usage(skill_id, skill_name))
    }

    /// Create a tool execution step
    pub fn tool_execution(
        task_id: TaskId,
        tool_name: impl Into<String>,
        tool_arguments: serde_json::Value,
    ) -> Self {
        Self::new(
            task_id,
            StepContent::tool_execution(tool_name, tool_arguments),
        )
    }

    /// Create a completion step
    pub fn completion(task_id: TaskId) -> Self {
        Self::new(task_id, StepContent::completion())
    }

    pub const fn step_type(&self) -> StepType {
        self.content.step_type()
    }

    pub fn title(&self) -> String {
        self.content.title()
    }

    pub fn tool_name(&self) -> Option<&str> {
        self.content.tool_name()
    }

    pub fn tool_arguments(&self) -> Option<&serde_json::Value> {
        self.content.tool_arguments()
    }

    pub fn tool_result(&self) -> Option<&serde_json::Value> {
        self.content.tool_result()
    }

    pub fn reasoning(&self) -> Option<&str> {
        self.content.reasoning()
    }

    pub fn complete(&mut self, result: Option<serde_json::Value>) {
        let now = Utc::now();
        self.status = StepStatus::Completed;
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as i32);
        if let Some(r) = result {
            self.content = self.content.clone().with_tool_result(r);
        }
    }

    pub fn fail(&mut self, error: String) {
        let now = Utc::now();
        self.status = StepStatus::Failed;
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as i32);
        self.error_message = Some(error);
    }
}

#[derive(Debug, Clone)]
pub struct TrackedStep {
    pub step_id: StepId,
    pub started_at: DateTime<Utc>,
}

pub type StepDetail = StepContent;
