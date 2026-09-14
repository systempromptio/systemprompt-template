-- One row per governed call: every evaluation that shared a trace, folded into
-- one row with its chain carried as JSON.
--
-- The predicate in `keys` and the one in `count.sql` are ONE predicate expressed
-- twice. `query_file!` verifies static text only, so a caller-supplied filter
-- cannot be spliced in and the compiler cannot prove the two agree — edit both
-- or the page and its total will disagree. `decision_calls_agree_with_their_count`
-- in the integration suite is the guard.
--
-- $1 from, $2 to, $3 subject scope, $4 policy, $5 decision, $6 search,
-- $7 sort key, $8 ascending, $9 limit, $10 offset, $11 attention only.
WITH scoped AS (
    SELECT g.id, g.created_at, g.decision, g.policy, g.reason, g.tool_name,
           g.user_id, g.agent_scope, g.trace_id,
           (g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')) AS actionable,
           g.evaluated_rules ->> 'entity_type' AS entity_type,
           CASE WHEN $11::BOOL AND (g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')) THEN 'review:' || md5(g.user_id || ':' || g.session_id || ':' || g.policy || ':' || COALESCE(substring(g.reason from 'fingerprint:([0-9a-f]{64})'), g.reason))
                ELSE COALESCE(NULLIF(g.trace_id, ''), g.id) END AS call_key
    FROM governance_decisions g
    WHERE g.created_at >= $1 AND g.created_at < $2
      AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
),
keys AS (
    SELECT s.call_key
    FROM scoped s
    WHERE ($4::TEXT IS NULL OR s.policy = $4)
      AND ($5::TEXT IS NULL OR s.decision = $5)
      AND ($6::TEXT IS NULL
           OR s.tool_name ILIKE '%' || $6 || '%'
           OR s.user_id ILIKE '%' || $6 || '%'
           OR s.reason ILIKE '%' || $6 || '%'
           OR s.call_key ILIKE '%' || $6 || '%')
    GROUP BY s.call_key
    HAVING NOT $11::BOOL OR bool_or(s.actionable)
),
member AS (
    SELECT s.*,
           CASE s.decision
               WHEN 'deny' THEN 3
               WHEN 'warn' THEN 2
               WHEN 'allow' THEN 1
               ELSE 0
           END AS sev
    FROM scoped s
    JOIN keys k ON k.call_key = s.call_key
),
worst AS (
    SELECT DISTINCT ON (m.call_key)
           m.call_key, m.id, m.policy, m.decision, m.reason,
           m.tool_name, m.entity_type, m.agent_scope, m.sev
    FROM member m
    ORDER BY m.call_key, m.sev DESC, m.created_at ASC, m.id ASC
),
grouped AS (
    SELECT m.call_key,
           MIN(m.created_at) AS started_at,
           MAX(m.created_at) AS ended_at,
           COUNT(*)::BIGINT AS eval_count,
           COUNT(*) FILTER (WHERE m.decision = 'deny')::BIGINT AS deny_count,
           COUNT(*) FILTER (WHERE m.decision = 'warn')::BIGINT AS warn_count,
           MIN(m.user_id) AS user_id,
           MAX(m.trace_id) AS trace_id,
           jsonb_agg(
               jsonb_build_object(
                   'id', m.id,
                   'policy', m.policy,
                   'decision', m.decision,
                   'reason', m.reason,
                   'tool_name', m.tool_name,
                   'entity_type', m.entity_type
               ) ORDER BY m.created_at, m.id
           ) AS chain
    FROM member m
    GROUP BY m.call_key
)
SELECT gr.call_key AS "call_key!",
       gr.started_at AS "started_at!",
       gr.ended_at AS "ended_at!",
       gr.eval_count AS "eval_count!",
       gr.deny_count AS "deny_count!",
       gr.warn_count AS "warn_count!",
       gr.user_id AS "user_id!: UserId",
       COALESCE(u.display_name, u.full_name, u.name, u.email, gr.user_id)
           AS "user_label!",
       gr.trace_id AS "trace_id?",
       w.id AS "worst_id!",
       w.decision AS "worst_decision!",
       w.policy AS "worst_policy!",
       w.reason AS "worst_reason!",
       w.tool_name AS "worst_tool!",
       w.entity_type AS "worst_entity_type?",
       w.agent_scope AS "agent_scope?",
       gr.chain AS "chain!: Json<Vec<ChainEvaluation>>"
FROM grouped gr
JOIN worst w ON w.call_key = gr.call_key
LEFT JOIN LATERAL (
    SELECT x.display_name, x.full_name, x.name, x.email
    FROM users x WHERE x.id = gr.user_id
) u ON true
ORDER BY
  CASE WHEN $7 = 'policy'   AND $8 THEN w.policy END ASC,
  CASE WHEN $7 = 'policy'   AND NOT $8 THEN w.policy END DESC,
  CASE WHEN $7 = 'decision' AND $8 THEN w.sev END ASC,
  CASE WHEN $7 = 'decision' AND NOT $8 THEN w.sev END DESC,
  CASE WHEN $7 = 'tool'     AND $8 THEN w.tool_name END ASC,
  CASE WHEN $7 = 'tool'     AND NOT $8 THEN w.tool_name END DESC,
  CASE WHEN $7 = 'user'     AND $8 THEN gr.user_id END ASC,
  CASE WHEN $7 = 'user'     AND NOT $8 THEN gr.user_id END DESC,
  CASE WHEN $8 THEN gr.ended_at END ASC,
  gr.ended_at DESC, gr.call_key DESC
LIMIT $9 OFFSET $10
