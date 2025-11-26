use crate::models::{ConversationEvaluation, ScheduledJob};
use chrono::{DateTime, NaiveDate, Utc};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

#[derive(Debug, Clone)]
pub struct SchedulerRepository {
    db_pool: DbPool,
}

impl SchedulerRepository {
    #[must_use]
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn upsert_job(
        &self,
        job_name: &str,
        schedule: &str,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let query = DatabaseQueryEnum::UpsertScheduledJob.get(self.db_pool.as_ref());
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        self.db_pool
            .execute(&query, &[&id, &job_name, &schedule, &enabled, &now, &now])
            .await?;

        Ok(())
    }

    pub async fn get_job(&self, job_name: &str) -> anyhow::Result<Option<ScheduledJob>> {
        let query = DatabaseQueryEnum::GetScheduledJob.get(self.db_pool.as_ref());
        let row = self.db_pool.fetch_optional(&query, &[&job_name]).await?;
        row.map(|r| ScheduledJob::from_json_row(&r)).transpose()
    }

    pub async fn list_enabled_jobs(&self) -> anyhow::Result<Vec<ScheduledJob>> {
        let query = DatabaseQueryEnum::ListEnabledJobs.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[]).await?;

        rows.into_iter()
            .map(|r| ScheduledJob::from_json_row(&r))
            .collect()
    }

    pub async fn update_job_execution(
        &self,
        job_name: &str,
        status: &str,
        error: Option<&str>,
        next_run: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        let query = DatabaseQueryEnum::UpdateJobExecution.get(self.db_pool.as_ref());
        let now = Utc::now();

        self.db_pool
            .execute(&query, &[&now, &status, &error, &next_run, &now, &job_name])
            .await?;

        Ok(())
    }

    pub async fn increment_run_count(&self, job_name: &str) -> anyhow::Result<()> {
        let query = DatabaseQueryEnum::IncrementJobRunCount.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[&job_name]).await?;
        Ok(())
    }

    pub async fn create_evaluation(&self, eval: &ConversationEvaluation) -> anyhow::Result<()> {
        let query = DatabaseQueryEnum::CreateEvaluation.get(self.db_pool.as_ref());

        let goal_achievement_confidence = eval.goal_achievement_confidence;
        let user_satisfied = eval.user_satisfied;
        let conversation_quality = eval.conversation_quality;
        let total_turns = eval.total_turns;
        let conversation_duration_seconds = eval.conversation_duration_seconds;
        let user_initiated = eval.user_initiated;
        let overall_score = eval.overall_score;

        self.db_pool
            .execute(
                &query,
                &[
                    &eval.context_id,
                    &eval.agent_goal,
                    &eval.goal_achieved,
                    &goal_achievement_confidence,
                    &eval.goal_achievement_notes.as_deref(),
                    &eval.primary_category,
                    &eval.topics_discussed,
                    &eval.keywords,
                    &user_satisfied,
                    &conversation_quality,
                    &eval.quality_notes.as_deref(),
                    &eval.issues_encountered.as_deref(),
                    &eval.agent_name,
                    &total_turns,
                    &conversation_duration_seconds,
                    &user_initiated,
                    &eval.completion_status,
                    &overall_score,
                    &eval.evaluation_summary,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_evaluation_by_context(
        &self,
        context_id: &str,
    ) -> anyhow::Result<Option<ConversationEvaluation>> {
        let query = DatabaseQueryEnum::GetEvaluationByContext.get(self.db_pool.as_ref());
        let row = self.db_pool.fetch_optional(&query, &[&context_id]).await?;
        row.map(|r| ConversationEvaluation::from_json_row(&r))
            .transpose()
    }

    pub async fn get_evaluation_metrics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetEvaluationMetrics.get(self.db_pool.as_ref());
        let start = start_date.to_string();
        let end = end_date.to_string();
        self.db_pool.fetch_all(&query, &[&start, &end]).await
    }

    pub async fn get_low_scoring_conversations(
        &self,
        score_threshold: f64,
        limit: i64,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetLowScoringConversations.get(self.db_pool.as_ref());
        self.db_pool
            .fetch_all(&query, &[&score_threshold, &limit])
            .await
    }

    pub async fn get_unevaluated_conversations(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetUnevaluatedConversations.get(self.db_pool.as_ref());
        self.db_pool.fetch_all(&query, &[&limit]).await
    }

    pub async fn get_top_issues_encountered(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetTopIssuesEncountered.get(self.db_pool.as_ref());
        self.db_pool
            .fetch_all(&query, &[&start_date, &end_date])
            .await
    }

    pub async fn get_goal_achievement_stats(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetGoalAchievementStats.get(self.db_pool.as_ref());
        self.db_pool
            .fetch_all(&query, &[&start_date, &end_date])
            .await
    }

    pub async fn get_detailed_evaluations(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetDetailedEvaluations.get(self.db_pool.as_ref());
        self.db_pool
            .fetch_all(&query, &[&start_date, &end_date, &limit, &offset])
            .await
    }

    pub async fn get_evaluation_quality_distribution(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> anyhow::Result<Vec<systemprompt_core_database::JsonRow>> {
        let query = DatabaseQueryEnum::GetEvaluationQualityDistribution.get(self.db_pool.as_ref());
        self.db_pool
            .fetch_all(&query, &[&start_date, &end_date])
            .await
    }
}
