use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Scheduler module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        UpsertScheduledJob => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/upsert_scheduled_job.sql"
        )),
        GetScheduledJob => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_scheduled_job.sql"
        )),
        ListEnabledJobs => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/list_enabled_jobs.sql"
        )),
        UpdateJobExecution => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/update_job_execution.sql"
        )),
        IncrementJobRunCount => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/increment_job_run_count.sql"
        )),
        CreateEvaluation => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/create_evaluation.sql"
        )),
        GetEvaluationByContext => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_evaluation_by_task.sql"
        )),
        GetEvaluationMetrics => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_evaluation_metrics.sql"
        )),
        GetLowScoringConversations => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_low_scoring_conversations.sql"
        )),
        GetEvaluationQualityDistribution => Some(include_str!(
            "../../../../scheduler/src/queries/evaluations/postgres/get_evaluation_quality_distribution.sql"
        )),
        GetRecentEvaluations => Some(include_str!(
            "../../../../scheduler/src/queries/evaluations/postgres/get_recent_evaluations.sql"
        )),
        GetConversationsByLocation => Some(include_str!(
            "../../../../scheduler/src/queries/evaluations/postgres/get_conversations_by_location.sql"
        )),
        GetUnevaluatedConversations => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_unevaluated_conversations.sql"
        )),
        GetTopIssuesEncountered => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_top_issues_encountered.sql"
        )),
        GetGoalAchievementStats => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_goal_achievement_stats.sql"
        )),
        GetDetailedEvaluations => Some(include_str!(
            "../../../../scheduler/src/queries/core/postgres/get_detailed_evaluations.sql"
        )),

        _ => None,
    }
}
