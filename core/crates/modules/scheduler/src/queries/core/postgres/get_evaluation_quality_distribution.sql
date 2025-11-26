-- Get distribution of conversation quality ratings (bucketed from numeric 0-100 scores)
-- Used by MCP admin tool for quality analytics
SELECT
    CASE
        WHEN conversation_quality >= 80 THEN 'excellent'
        WHEN conversation_quality >= 60 THEN 'good'
        WHEN conversation_quality >= 40 THEN 'acceptable'
        WHEN conversation_quality >= 20 THEN 'poor'
        ELSE 'very_poor'
    END as quality_bucket,
    COUNT(*) as count,
    ROUND(COUNT(*)::NUMERIC / NULLIF(SUM(COUNT(*)) OVER (), 0) * 100, 1) as percentage,
    AVG(overall_score) as avg_score,
    AVG(conversation_quality) as avg_quality_score
FROM conversation_evaluations
WHERE analyzed_at >= $1 AND analyzed_at <= $2
GROUP BY quality_bucket
ORDER BY
    CASE quality_bucket
        WHEN 'excellent' THEN 1
        WHEN 'good' THEN 2
        WHEN 'acceptable' THEN 3
        WHEN 'poor' THEN 4
        WHEN 'very_poor' THEN 5
    END;
