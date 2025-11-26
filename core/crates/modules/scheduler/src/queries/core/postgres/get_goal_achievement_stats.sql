-- Get goal achievement statistics
-- Used by MCP admin tool for goal analytics
SELECT
    goal_achieved,
    COUNT(*) as count,
    ROUND(COUNT(*)::NUMERIC / NULLIF(SUM(COUNT(*)) OVER (), 0) * 100, 1) as percentage,
    ROUND(AVG(goal_achievement_confidence)::NUMERIC, 3) as avg_confidence,
    ROUND(AVG(overall_score)::NUMERIC, 3) as avg_overall_score
FROM conversation_evaluations
WHERE analyzed_at >= $1 AND analyzed_at <= $2
GROUP BY goal_achieved
ORDER BY
    CASE goal_achieved
        WHEN 'yes' THEN 1
        WHEN 'partial' THEN 2
        WHEN 'no' THEN 3
        ELSE 4
    END;
