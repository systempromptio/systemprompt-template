-- Get most frequently encountered issues in conversations
-- Used by MCP admin tool for issue analytics
SELECT
    TRIM(issue) as issue,
    COUNT(*) as frequency,
    ROUND(COUNT(*)::NUMERIC / NULLIF((SELECT COUNT(*) FROM conversation_evaluations WHERE analyzed_at >= $1 AND analyzed_at <= $2 AND issues_encountered IS NOT NULL AND issues_encountered != ''), 0) * 100, 1) as percentage
FROM conversation_evaluations,
LATERAL unnest(string_to_array(issues_encountered, ',')) as issue
WHERE analyzed_at >= $1
  AND analyzed_at <= $2
  AND issues_encountered IS NOT NULL
  AND issues_encountered != ''
  AND TRIM(issue) != ''
GROUP BY TRIM(issue)
ORDER BY frequency DESC
LIMIT 10;
