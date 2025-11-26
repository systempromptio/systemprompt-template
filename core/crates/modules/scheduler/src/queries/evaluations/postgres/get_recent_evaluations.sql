SELECT
    context_id,
    primary_category,
    overall_score as quality_score,
    user_satisfied,
    keywords,
    evaluation_summary as summary,
    analyzed_at
FROM conversation_evaluations
WHERE analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
ORDER BY analyzed_at DESC
LIMIT $2
