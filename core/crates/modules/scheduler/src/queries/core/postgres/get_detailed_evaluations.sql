-- Get detailed evaluation data for display in MCP table
-- Used by MCP admin tool for detailed conversation analysis
SELECT
    e.id,
    e.context_id,
    e.agent_name,
    e.agent_goal,
    e.goal_achieved,
    e.goal_achievement_confidence,
    e.primary_category,
    e.topics_discussed,
    e.keywords,
    e.user_satisfied,
    e.conversation_quality,
    e.quality_notes,
    e.issues_encountered,
    e.total_turns,
    e.conversation_duration_seconds,
    e.completion_status,
    e.overall_score,
    e.evaluation_summary,
    e.analyzed_at,
    e.analysis_version,
    MIN(t.started_at) as conversation_started_at,
    MAX(t.completed_at) as conversation_completed_at
FROM conversation_evaluations e
JOIN agent_tasks t ON e.context_id = t.context_id
WHERE e.analyzed_at >= $1 AND e.analyzed_at <= $2
GROUP BY e.id, e.context_id, e.agent_name, e.agent_goal, e.goal_achieved, e.goal_achievement_confidence,
         e.primary_category, e.topics_discussed, e.keywords, e.user_satisfied, e.conversation_quality,
         e.quality_notes, e.issues_encountered, e.total_turns, e.conversation_duration_seconds,
         e.completion_status, e.overall_score, e.evaluation_summary, e.analyzed_at, e.analysis_version
ORDER BY e.analyzed_at DESC
LIMIT $3
OFFSET $4;
