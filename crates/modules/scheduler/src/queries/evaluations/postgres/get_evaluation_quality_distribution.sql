SELECT
    COUNT(CASE WHEN overall_score > 0.8 THEN 1 END) as excellent,
    COUNT(CASE WHEN overall_score > 0.6 AND overall_score <= 0.8 THEN 1 END) as good,
    COUNT(CASE WHEN overall_score > 0.4 AND overall_score <= 0.6 THEN 1 END) as fair,
    COUNT(CASE WHEN overall_score <= 0.4 THEN 1 END) as poor
FROM conversation_evaluations
WHERE analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
