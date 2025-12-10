use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use systemprompt_identifiers::{SkillId, TaskId};
use systemprompt_models::{ExecutionStep, PlannedTool, StepContent, StepId, TrackedStep};

use crate::repository::ExecutionStepRepository;

#[derive(Debug, Clone)]
pub struct ExecutionTrackingService {
    repository: Arc<ExecutionStepRepository>,
}

impl ExecutionTrackingService {
    pub const fn new(repository: Arc<ExecutionStepRepository>) -> Self {
        Self { repository }
    }

    pub async fn track(&self, task_id: TaskId, content: StepContent) -> Result<ExecutionStep> {
        let step = ExecutionStep::new(task_id, content);
        self.repository.create(&step).await?;
        Ok(step)
    }

    pub async fn track_async(
        &self,
        task_id: TaskId,
        content: StepContent,
    ) -> Result<(TrackedStep, ExecutionStep)> {
        let step = ExecutionStep::new(task_id, content);
        self.repository.create(&step).await?;

        let tracked = TrackedStep {
            step_id: step.step_id.clone(),
            started_at: step.started_at,
        };

        Ok((tracked, step))
    }

    pub async fn complete(
        &self,
        tracked: TrackedStep,
        result: Option<serde_json::Value>,
    ) -> Result<()> {
        self.repository
            .complete_step(&tracked.step_id, tracked.started_at, result)
            .await
    }

    pub async fn complete_planning(
        &self,
        tracked: TrackedStep,
        reasoning: Option<String>,
        planned_tools: Option<Vec<PlannedTool>>,
    ) -> Result<ExecutionStep> {
        self.repository
            .complete_planning_step(
                &tracked.step_id,
                tracked.started_at,
                reasoning,
                planned_tools,
            )
            .await
    }

    pub async fn fail(&self, tracked: &TrackedStep, error: String) -> Result<()> {
        self.repository
            .fail_step(&tracked.step_id, tracked.started_at, &error)
            .await
    }

    pub async fn fail_step(
        &self,
        step_id: &StepId,
        started_at: DateTime<Utc>,
        error: String,
    ) -> Result<()> {
        self.repository.fail_step(step_id, started_at, &error).await
    }

    pub async fn get_steps_by_task(&self, task_id: &str) -> Result<Vec<ExecutionStep>> {
        self.repository.list_by_task(task_id).await
    }

    pub async fn get_step(&self, step_id: &StepId) -> Result<Option<ExecutionStep>> {
        self.repository.get(step_id).await
    }

    pub async fn fail_in_progress_steps(&self, task_id: &str, error: &str) -> Result<u64> {
        self.repository
            .fail_in_progress_steps_for_task(task_id, error)
            .await
    }

    /// Track an understanding step (instant)
    pub async fn track_understanding(&self, task_id: TaskId) -> Result<ExecutionStep> {
        self.track(task_id, StepContent::understanding()).await
    }

    /// Track a planning step (instant)
    pub async fn track_planning(
        &self,
        task_id: TaskId,
        reasoning: Option<String>,
        planned_tools: Option<Vec<PlannedTool>>,
    ) -> Result<ExecutionStep> {
        self.track(task_id, StepContent::planning(reasoning, planned_tools))
            .await
    }

    /// Track a planning step asynchronously (returns TrackedStep for
    /// completion)
    pub async fn track_planning_async(
        &self,
        task_id: TaskId,
        reasoning: Option<String>,
        planned_tools: Option<Vec<PlannedTool>>,
    ) -> Result<(TrackedStep, ExecutionStep)> {
        self.track_async(task_id, StepContent::planning(reasoning, planned_tools))
            .await
    }

    /// Track a skill usage step (instant)
    pub async fn track_skill_usage(
        &self,
        task_id: TaskId,
        skill_id: SkillId,
        skill_name: impl Into<String>,
    ) -> Result<ExecutionStep> {
        self.track(task_id, StepContent::skill_usage(skill_id, skill_name))
            .await
    }

    /// Track a tool execution step (async - returns TrackedStep for completion)
    pub async fn track_tool_execution(
        &self,
        task_id: TaskId,
        tool_name: impl Into<String>,
        tool_arguments: serde_json::Value,
    ) -> Result<(TrackedStep, ExecutionStep)> {
        self.track_async(
            task_id,
            StepContent::tool_execution(tool_name, tool_arguments),
        )
        .await
    }

    /// Track a completion step (instant)
    pub async fn track_completion(&self, task_id: TaskId) -> Result<ExecutionStep> {
        self.track(task_id, StepContent::completion()).await
    }
}
