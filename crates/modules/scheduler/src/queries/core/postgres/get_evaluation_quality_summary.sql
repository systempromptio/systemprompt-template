SELECT
    COUNT(*) as total_evaluations,
    AVG(overall_score) as avg_overall_score,
    COUNT(*) FILTER (WHERE user_satisfied = 'yes') as satisfied_count,
    COUNT(*) FILTER (WHERE user_satisfied = 'unclear') as unclear_count,
    COUNT(*) FILTER (WHERE user_satisfied = 'no') as dissatisfied_count
FROM conversation_evaluations
WHERE analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
