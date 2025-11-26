SELECT
    context_id, overall_score, conversation_quality, user_satisfied,
    completion_status, evaluation_summary, quality_notes,
    analyzed_at
FROM conversation_evaluations
WHERE overall_score < $1
ORDER BY overall_score ASC, analyzed_at DESC
LIMIT $2
