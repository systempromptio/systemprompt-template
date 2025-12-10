use crate::models::{ConversationEvaluation, ScheduledJob};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

#[derive(Debug, Clone)]
pub struct SchedulerRepository {
    pool: Arc<PgPool>,
}

impl SchedulerRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn upsert_job(
        &self,
        job_name: &str,
        schedule: &str,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO scheduled_jobs (id, job_name, schedule, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(job_name) DO UPDATE SET
                schedule = EXCLUDED.schedule,
                enabled = EXCLUDED.enabled,
                updated_at = EXCLUDED.updated_at
            "#,
            id,
            job_name,
            schedule,
            enabled,
            now,
            now
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_job(&self, job_name: &str) -> anyhow::Result<Option<ScheduledJob>> {
        sqlx::query_as!(
            ScheduledJob,
            r#"
            SELECT id, job_name, schedule, enabled, last_run, next_run, last_status, last_error,
                   run_count, created_at, updated_at
            FROM scheduled_jobs
            WHERE job_name = $1
            "#,
            job_name
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn list_enabled_jobs(&self) -> anyhow::Result<Vec<ScheduledJob>> {
        sqlx::query_as!(
            ScheduledJob,
            r#"
            SELECT id, job_name, schedule, enabled, last_run, next_run, last_status, last_error,
                   run_count, created_at, updated_at
            FROM scheduled_jobs
            WHERE enabled = true
            ORDER BY job_name
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update_job_execution(
        &self,
        job_name: &str,
        status: &str,
        error: Option<&str>,
        next_run: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        let now = Utc::now();

        sqlx::query!(
            r#"
            UPDATE scheduled_jobs
            SET last_run = $1,
                last_status = $2,
                last_error = $3,
                next_run = $4,
                updated_at = $5
            WHERE job_name = $6
            "#,
            now,
            status,
            error,
            next_run,
            now,
            job_name
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn increment_run_count(&self, job_name: &str) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE scheduled_jobs SET run_count = run_count + 1 WHERE job_name = $1",
            job_name
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_evaluation(&self, eval: &ConversationEvaluation) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO conversation_evaluations (
                context_id, agent_goal, goal_achieved, goal_achievement_confidence,
                goal_achievement_notes, primary_category, topics_discussed, keywords,
                user_satisfied, conversation_quality, quality_notes, issues_encountered,
                agent_name, total_turns, conversation_duration_seconds, user_initiated,
                completion_status, overall_score, evaluation_summary, analysis_version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            "#,
            eval.context_id,
            eval.agent_goal,
            eval.goal_achieved,
            eval.goal_achievement_confidence,
            eval.goal_achievement_notes,
            eval.primary_category,
            eval.topics_discussed,
            eval.keywords,
            eval.user_satisfied,
            eval.conversation_quality,
            eval.quality_notes,
            eval.issues_encountered,
            eval.agent_name,
            eval.total_turns,
            eval.conversation_duration_seconds,
            eval.user_initiated,
            eval.completion_status,
            eval.overall_score,
            eval.evaluation_summary,
            eval.analysis_version
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_evaluation_by_context(
        &self,
        context_id: &str,
    ) -> anyhow::Result<Option<ConversationEvaluation>> {
        sqlx::query_as!(
            ConversationEvaluation,
            r#"
            SELECT id, context_id, agent_goal, goal_achieved,
                   goal_achievement_confidence, goal_achievement_notes,
                   primary_category, topics_discussed, keywords,
                   user_satisfied, conversation_quality, quality_notes, issues_encountered,
                   agent_name, total_turns, conversation_duration_seconds, user_initiated,
                   completion_status, overall_score,
                   evaluation_summary, analyzed_at, analysis_version
            FROM conversation_evaluations
            WHERE context_id = $1
            LIMIT 1
            "#,
            context_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_evaluation_metrics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                primary_category,
                COUNT(*) as "count!: i64",
                AVG(overall_score)::float8 as "avg_score: f64",
                MIN(overall_score)::float8 as "min_score: f64",
                MAX(overall_score)::float8 as "max_score: f64"
            FROM conversation_evaluations
            WHERE DATE(analyzed_at) >= $1 AND DATE(analyzed_at) <= $2
            GROUP BY primary_category
            "#,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "evaluation_type": row.primary_category,
                    "count": row.count,
                    "avg_score": row.avg_score,
                    "min_score": row.min_score,
                    "max_score": row.max_score,
                })
            })
            .collect())
    }

    pub async fn get_low_scoring_conversations(
        &self,
        score_threshold: f64,
        limit: i64,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let threshold = score_threshold as f32;
        let rows = sqlx::query!(
            r#"
            SELECT id, context_id, overall_score, evaluation_summary, analyzed_at
            FROM conversation_evaluations
            WHERE overall_score < $1
            ORDER BY overall_score ASC
            LIMIT $2
            "#,
            threshold,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "context_id": row.context_id,
                    "overall_score": row.overall_score,
                    "evaluation_summary": row.evaluation_summary,
                    "analyzed_at": row.analyzed_at,
                })
            })
            .collect())
    }

    pub async fn get_unevaluated_conversations(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT c.context_id, c.created_at
            FROM user_contexts c
            LEFT JOIN conversation_evaluations e ON c.context_id = e.context_id
            WHERE e.id IS NULL
              AND EXISTS (
                SELECT 1 FROM task_messages m
                JOIN agent_tasks t ON m.task_id = t.task_id
                WHERE t.context_id = c.context_id
              )
            ORDER BY c.created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "context_id": row.context_id,
                    "created_at": row.created_at,
                })
            })
            .collect())
    }

    pub async fn cleanup_empty_contexts(&self, hours_old: i64) -> anyhow::Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_contexts c
            WHERE NOT EXISTS (
                SELECT 1 FROM task_messages m
                JOIN agent_tasks t ON m.task_id = t.task_id
                WHERE t.context_id = c.context_id
            )
            AND c.created_at < NOW() - ($1 || ' hours')::INTERVAL
            "#,
            hours_old.to_string()
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_top_issues_encountered(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query!(
            r#"
            SELECT issues_encountered, COUNT(*) as "count!: i64"
            FROM conversation_evaluations
            WHERE issues_encountered IS NOT NULL
                AND analyzed_at >= $1
                AND analyzed_at <= $2
            GROUP BY issues_encountered
            ORDER BY 2 DESC
            LIMIT 20
            "#,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "issue": row.issues_encountered,
                    "count": row.count,
                })
            })
            .collect())
    }

    pub async fn get_goal_achievement_stats(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query!(
            r#"
            SELECT goal_achievement_confidence, COUNT(*) as "count!: i64"
            FROM conversation_evaluations
            WHERE analyzed_at >= $1 AND analyzed_at <= $2
            GROUP BY goal_achievement_confidence
            ORDER BY goal_achievement_confidence
            "#,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "confidence": row.goal_achievement_confidence,
                    "count": row.count,
                })
            })
            .collect())
    }

    pub async fn get_detailed_evaluations(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ConversationEvaluation>> {
        sqlx::query_as!(
            ConversationEvaluation,
            r#"
            SELECT id, context_id, agent_goal, goal_achieved,
                   goal_achievement_confidence, goal_achievement_notes,
                   primary_category, topics_discussed, keywords,
                   user_satisfied, conversation_quality, quality_notes, issues_encountered,
                   agent_name, total_turns, conversation_duration_seconds, user_initiated,
                   completion_status, overall_score,
                   evaluation_summary, analyzed_at, analysis_version
            FROM conversation_evaluations
            WHERE analyzed_at >= $1 AND analyzed_at <= $2
            ORDER BY analyzed_at DESC
            LIMIT $3 OFFSET $4
            "#,
            start_date,
            end_date,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_evaluation_quality_distribution(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                CASE
                    WHEN overall_score >= 0.9 THEN 'excellent'
                    WHEN overall_score >= 0.7 THEN 'good'
                    WHEN overall_score >= 0.5 THEN 'fair'
                    ELSE 'poor'
                END as quality_bucket,
                COUNT(*) as "count!: i64"
            FROM conversation_evaluations
            WHERE analyzed_at >= $1 AND analyzed_at <= $2
            GROUP BY quality_bucket
            "#,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "quality_bucket": row.quality_bucket,
                    "count": row.count,
                })
            })
            .collect())
    }
}
