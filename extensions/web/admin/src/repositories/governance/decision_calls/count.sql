-- How many calls the page's filter selected. See the warning in page.sql: this
-- predicate and the one in that file's `keys` CTE are one predicate, and nothing
-- but a test enforces that they agree.
--
-- $1 from, $2 to, $3 subject scope, $4 policy, $5 decision, $6 search,
-- $7 attention only.
SELECT COUNT(*)::BIGINT AS "n!"
FROM (
    SELECT CASE WHEN $7::BOOL AND (g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')) THEN 'review:' || md5(g.user_id || ':' || g.session_id || ':' || g.policy || ':' || COALESCE(substring(g.reason from 'fingerprint:([0-9a-f]{64})'), g.reason))
                ELSE COALESCE(NULLIF(g.trace_id, ''), g.id) END AS call_key
    FROM governance_decisions g
    WHERE g.created_at >= $1 AND g.created_at < $2
      AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
      AND ($4::TEXT IS NULL OR g.policy = $4)
      AND ($5::TEXT IS NULL OR g.decision = $5)
      AND ($6::TEXT IS NULL
           OR g.tool_name ILIKE '%' || $6 || '%'
           OR g.user_id ILIKE '%' || $6 || '%'
           OR g.reason ILIKE '%' || $6 || '%'
           OR COALESCE(NULLIF(g.trace_id, ''), g.id) ILIKE '%' || $6 || '%')
    GROUP BY 1
    HAVING NOT $7::BOOL OR bool_or((g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')))
) k
