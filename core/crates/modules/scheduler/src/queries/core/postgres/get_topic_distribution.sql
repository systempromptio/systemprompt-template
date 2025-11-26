SELECT
    primary_category as topic,
    COUNT(*) as conversation_count,
    AVG(overall_score) as avg_quality_score
FROM conversation_evaluations
WHERE analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY primary_category
ORDER BY conversation_count DESC
