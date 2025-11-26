-- Get evaluation metrics for conversations created within date range
-- Returns individual conversation evaluations (not aggregated)
-- Filters by conversation creation date, not evaluation analysis date
SELECT
    ce.context_id,
    ce.conversation_quality,
    ce.goal_achieved,
    ce.user_satisfied,
    ce.overall_score,
    ce.analyzed_at
FROM conversation_evaluations ce
INNER JOIN user_contexts uc ON ce.context_id = uc.context_id
WHERE uc.created_at >= $1 AND uc.created_at <= $2
ORDER BY ce.analyzed_at DESC
