use crate::models::{ExecutionStep, PlannedTool, StepContent, StepId, StepStatus};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

#[derive(Debug, Clone)]
pub struct ExecutionStepRepository {
    pool: Arc<PgPool>,
}

impl ExecutionStepRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn create(&self, step: &ExecutionStep) -> Result<()> {
        let step_id_str = step.step_id.as_str();
        let task_id = &step.task_id;
        let status_str = step.status.to_string();
        let content_json =
            serde_json::to_value(&step.content).context("Failed to serialize step content")?;

        sqlx::query!(
            r#"INSERT INTO task_execution_steps (
                step_id, task_id, status, content, started_at, completed_at, duration_ms, error_message
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            step_id_str,
            task_id,
            status_str,
            content_json,
            step.started_at,
            step.completed_at,
            step.duration_ms,
            step.error_message
        )
        .execute(&*self.pool)
        .await
        .context("Failed to create execution step")?;

        Ok(())
    }

    pub async fn get(&self, step_id: &StepId) -> Result<Option<ExecutionStep>> {
        let step_id_str = step_id.as_str();

        let row = sqlx::query!(
            r#"SELECT step_id, task_id, status, content,
                    started_at as "started_at!", completed_at, duration_ms, error_message
                FROM task_execution_steps WHERE step_id = $1"#,
            step_id_str
        )
        .fetch_optional(&*self.pool)
        .await
        .context(format!("Failed to get execution step: {step_id}"))?;

        row.map(|r| {
            let status = r
                .status
                .parse::<StepStatus>()
                .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?;

            let content: StepContent = serde_json::from_value(r.content)
                .map_err(|e| anyhow::anyhow!("Invalid content: {}", e))?;

            Ok(ExecutionStep {
                step_id: r.step_id.into(),
                task_id: r.task_id,
                status,
                started_at: r.started_at,
                completed_at: r.completed_at,
                duration_ms: r.duration_ms,
                error_message: r.error_message,
                content,
            })
        })
        .transpose()
    }

    pub async fn list_by_task(&self, task_id: &str) -> Result<Vec<ExecutionStep>> {
        let rows = sqlx::query!(
            r#"SELECT step_id, task_id, status, content,
                    started_at as "started_at!", completed_at, duration_ms, error_message
                FROM task_execution_steps WHERE task_id = $1 ORDER BY started_at ASC"#,
            task_id
        )
        .fetch_all(&*self.pool)
        .await
        .context(format!(
            "Failed to list execution steps for task: {}",
            task_id
        ))?;

        rows.into_iter()
            .map(|r| {
                let status = r
                    .status
                    .parse::<StepStatus>()
                    .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?;

                let content: StepContent = serde_json::from_value(r.content)
                    .map_err(|e| anyhow::anyhow!("Invalid content: {}", e))?;

                Ok(ExecutionStep {
                    step_id: r.step_id.into(),
                    task_id: r.task_id,
                    status,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    duration_ms: r.duration_ms,
                    error_message: r.error_message,
                    content,
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn complete_step(
        &self,
        step_id: &StepId,
        started_at: DateTime<Utc>,
        tool_result: Option<serde_json::Value>,
    ) -> Result<()> {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as i32;
        let step_id_str = step_id.as_str();
        let status_str = StepStatus::Completed.to_string();

        if let Some(result) = tool_result {
            sqlx::query!(
                r#"UPDATE task_execution_steps SET
                    status = $2,
                    completed_at = $3,
                    duration_ms = $4,
                    content = jsonb_set(content, '{tool_result}', $5),
                    updated_at = CURRENT_TIMESTAMP
                WHERE step_id = $1"#,
                step_id_str,
                status_str,
                completed_at,
                duration_ms,
                result
            )
            .execute(&*self.pool)
            .await
            .context(format!("Failed to complete execution step: {step_id}"))?;
        } else {
            sqlx::query!(
                r#"UPDATE task_execution_steps SET
                    status = $2,
                    completed_at = $3,
                    duration_ms = $4,
                    updated_at = CURRENT_TIMESTAMP
                WHERE step_id = $1"#,
                step_id_str,
                status_str,
                completed_at,
                duration_ms
            )
            .execute(&*self.pool)
            .await
            .context(format!("Failed to complete execution step: {step_id}"))?;
        }

        Ok(())
    }

    pub async fn fail_step(
        &self,
        step_id: &StepId,
        started_at: DateTime<Utc>,
        error_message: &str,
    ) -> Result<()> {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as i32;
        let step_id_str = step_id.as_str();
        let status_str = StepStatus::Failed.to_string();

        sqlx::query!(
            r#"UPDATE task_execution_steps SET
                status = $2,
                completed_at = $3,
                duration_ms = $4,
                error_message = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE step_id = $1"#,
            step_id_str,
            status_str,
            completed_at,
            duration_ms,
            error_message
        )
        .execute(&*self.pool)
        .await
        .context(format!("Failed to fail execution step: {step_id}"))?;

        Ok(())
    }

    pub async fn fail_in_progress_steps_for_task(
        &self,
        task_id: &str,
        error_message: &str,
    ) -> Result<u64> {
        let completed_at = Utc::now();
        let in_progress_str = StepStatus::InProgress.to_string();
        let failed_str = StepStatus::Failed.to_string();

        let result = sqlx::query!(
            r#"UPDATE task_execution_steps SET
                status = $3,
                completed_at = $4,
                error_message = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE task_id = $1 AND status = $2"#,
            task_id,
            in_progress_str,
            failed_str,
            completed_at,
            error_message
        )
        .execute(&*self.pool)
        .await
        .context(format!(
            "Failed to fail in-progress steps for task: {}",
            task_id
        ))?;

        Ok(result.rows_affected())
    }

    pub async fn complete_planning_step(
        &self,
        step_id: &StepId,
        started_at: DateTime<Utc>,
        reasoning: Option<String>,
        planned_tools: Option<Vec<PlannedTool>>,
    ) -> Result<ExecutionStep> {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as i32;
        let step_id_str = step_id.as_str();
        let status_str = StepStatus::Completed.to_string();

        let content = StepContent::planning(reasoning, planned_tools);
        let content_json =
            serde_json::to_value(&content).context("Failed to serialize planning content")?;

        let row = sqlx::query!(
            r#"UPDATE task_execution_steps SET
                status = $2,
                completed_at = $3,
                duration_ms = $4,
                content = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE step_id = $1
            RETURNING step_id, task_id, status, content,
                    started_at as "started_at!", completed_at, duration_ms, error_message"#,
            step_id_str,
            status_str,
            completed_at,
            duration_ms,
            content_json
        )
        .fetch_one(&*self.pool)
        .await
        .context(format!("Failed to complete planning step: {step_id}"))?;

        let status = row
            .status
            .parse::<StepStatus>()
            .map_err(|e| anyhow::anyhow!("Invalid status: {}", e))?;

        let content: StepContent = serde_json::from_value(row.content)
            .map_err(|e| anyhow::anyhow!("Invalid content: {}", e))?;

        Ok(ExecutionStep {
            step_id: row.step_id.into(),
            task_id: row.task_id,
            status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            duration_ms: row.duration_ms,
            error_message: row.error_message,
            content,
        })
    }
}
