-- One row per skill run: a harness session that invoked a skill of the
-- marketplace (or the one skill) in the window, measured from its first
-- invocation to the agent's last stop. A session can span several contexts
-- (subagents), so the fact columns sum over all of them and the row links to
-- the context with the most turns.
--
-- Time splits into agent time and human wait. A prompt's wait is the gap
-- since the agent last stopped; a question's wait is the gap between the
-- model's answer that asked it and the AskUserQuestion result.
--
-- A skill that logs its workflow to `.sf-dev-workflow/<key>/workflow.log`
-- (or `audit.log`, the releases before it) reports the state it reached and
-- wall time per phase, parsed from the Bash commands that wrote the log.
WITH inv AS (
    SELECT e.session_id, e.skill, e.invoked_at,
           (SELECT o.marketplace_id FROM service_owned_ids o
             WHERE o.kind = 'plugin' AND o.id = e.plugin_id LIMIT 1) AS marketplace_id
    FROM analysis_skill_version_events e
    WHERE e.invoked_at >= $1 AND e.invoked_at < $2
      AND ($4::text IS NULL OR e.skill = $4)
      AND ($5::text[] IS NULL OR e.user_id = ANY($5))
),
runs AS (
    SELECT DISTINCT ON (i.session_id) i.session_id, i.skill, i.marketplace_id, i.invoked_at AS started_at
    FROM inv i
    WHERE $3::text IS NULL OR i.marketplace_id = $3
    ORDER BY i.session_id, i.invoked_at
),
picked AS (
    SELECT r.session_id, r.skill, r.marketplace_id, r.started_at,
           (ARRAY_AGG(f.context_id ORDER BY f.turn_count DESC, f.first_at))[1] AS context_id,
           MAX(f.last_at) AS last_at,
           SUM(f.turn_count)::bigint AS turns,
           SUM(f.tool_calls_intended)::bigint AS tool_calls,
           SUM(f.tool_calls_failed)::bigint AS tool_failures,
           SUM(f.error_count)::bigint AS errors,
           SUM(f.rejected_count)::bigint AS rejected,
           SUM(f.cost_microdollars)::bigint AS cost_microdollars,
           COUNT(*)::bigint AS contexts
    FROM runs r
    JOIN conversation_facts f ON f.client_session_id = r.session_id
    GROUP BY r.session_id, r.skill, r.marketplace_id, r.started_at
    ORDER BY r.started_at DESC
    LIMIT $6
),
bounds AS (
    SELECT p.session_id,
           GREATEST(p.last_at, s.last_stop, p.started_at) AS ended_at
    FROM picked p
    LEFT JOIN LATERAL (SELECT MAX(x.created_at) AS last_stop FROM plugin_usage_events x
                       WHERE x.session_id = p.session_id AND x.event_type = 'Stop'
                         AND x.created_at >= p.started_at) s ON true
),
prompts AS (
    SELECT p.session_id, p.started_at, u.created_at,
           LAG(u.created_at) OVER (PARTITION BY p.session_id ORDER BY u.created_at) AS prev_at
    FROM picked p
    JOIN plugin_usage_events u ON u.session_id = p.session_id
    WHERE u.event_type = 'UserPromptSubmit' AND COALESCE(u.metadata->>'agent_id', '') = ''
      AND u.created_at >= p.started_at - interval '2 seconds'
),
prompt_waits AS (
    SELECT q.session_id, COUNT(*)::bigint AS prompts,
           COALESCE(SUM(EXTRACT(EPOCH FROM q.created_at - s.stop_at)), 0)::float8 AS wait_s
    FROM prompts q
    LEFT JOIN LATERAL (SELECT MAX(x.created_at) AS stop_at FROM plugin_usage_events x
                       WHERE x.session_id = q.session_id AND x.event_type = 'Stop'
                         AND x.created_at >= q.started_at AND x.created_at < q.created_at
                         AND x.created_at > COALESCE(q.prev_at, '-infinity'::timestamptz)) s ON true
    GROUP BY q.session_id
),
question_waits AS (
    SELECT p.session_id, COUNT(*)::bigint AS questions,
           COALESCE(SUM(EXTRACT(EPOCH FROM a.created_at - asked.asked_at)), 0)::float8 AS wait_s
    FROM picked p
    JOIN plugin_usage_events a ON a.session_id = p.session_id
    JOIN LATERAL (SELECT MAX(cr.created_at + make_interval(secs => COALESCE(cr.latency_ms, 0) / 1000.0)) AS asked_at
                  FROM conversation_facts f
                  JOIN conversation_requests cr ON cr.context_id = f.context_id
                  WHERE f.client_session_id = p.session_id AND cr.created_at < a.created_at) asked ON true
    WHERE a.event_type IN ('PostToolUse', 'PostToolUseFailure') AND a.tool_name = 'AskUserQuestion'
      AND a.created_at >= p.started_at AND asked.asked_at IS NOT NULL AND asked.asked_at < a.created_at
    GROUP BY p.session_id
),
log_commands AS (
    SELECT p.session_id, u.created_at AS at, tool_input_summary(x.input) AS command
    FROM picked p
    JOIN plugin_usage_events u ON u.session_id = p.session_id
    JOIN mcp_tool_executions x ON x.ai_tool_call_id = u.metadata->>'tool_use_id' AND x.user_id = u.user_id
    WHERE u.event_type = 'PostToolUse' AND u.tool_name = 'Bash' AND u.created_at >= p.started_at
      AND x.input LIKE '%.sf-dev-workflow/%'
),
milestones AS (
    SELECT c.session_id, c.at, m.ord, m.parts[2] AS to_state
    FROM log_commands c
    CROSS JOIN LATERAL regexp_matches(c.command,
        '\$\(date[^)]*\)"[\s\\]+([A-Z][A-Z_]+)[\s\\]+([A-Z][A-Z_]+)', 'g') WITH ORDINALITY AS m(parts, ord)
    UNION ALL
    SELECT c.session_id, c.at, 1000 + m.ord, m.parts[2]
    FROM log_commands c
    CROSS JOIN LATERAL regexp_matches(c.command, '([A-Z][A-Z_]{2,}) -> ([A-Z][A-Z_]{2,})', 'g')
        WITH ORDINALITY AS m(parts, ord)
    WHERE c.command LIKE '%audit.log%'
),
steps AS (
    SELECT m.session_id, m.to_state, m.at,
           LEAD(m.at) OVER w AS next_at,
           ROW_NUMBER() OVER (PARTITION BY m.session_id ORDER BY m.at DESC, m.ord DESC) AS from_end
    FROM milestones m
    WINDOW w AS (PARTITION BY m.session_id ORDER BY m.at, m.ord)
),
phases AS (
    SELECT s.session_id,
           MAX(s.to_state) FILTER (WHERE s.from_end = 1) AS terminal_state,
           COUNT(*)::bigint AS transitions,
           SUM(d.secs) FILTER (WHERE d.phase = 'requirements')::float8 AS requirements_s,
           SUM(d.secs) FILTER (WHERE d.phase = 'design')::float8 AS design_s,
           SUM(d.secs) FILTER (WHERE d.phase = 'build')::float8 AS build_s,
           SUM(d.secs) FILTER (WHERE d.phase = 'verify')::float8 AS verify_s,
           SUM(d.secs) FILTER (WHERE d.phase = 'ship')::float8 AS ship_s
    FROM steps s
    JOIN bounds b ON b.session_id = s.session_id
    CROSS JOIN LATERAL (SELECT
        CASE
            WHEN s.to_state IN ('PR_CREATED', 'STOPPED', 'POLICY_REJECTED') THEN NULL
            WHEN s.to_state LIKE 'AWAITING_SHIP%' OR s.to_state = 'PR_READY' THEN 'ship'
            WHEN s.to_state IN ('VERIFICATION', 'FINAL_DEVELOPER_REVIEW', 'AI_CODE_REVIEW') THEN 'verify'
            WHEN s.to_state LIKE 'REQUIREMENT%' OR s.to_state LIKE 'AWAITING_REQUIREMENT%'
                 OR s.to_state LIKE 'CLARIFICATION%' OR s.to_state IN ('REQUEST_RECEIVED', 'POLICY_CHECK', 'JIRA_CONTEXT_REQUIRED') THEN 'requirements'
            WHEN s.to_state LIKE 'SOLUTION%' OR s.to_state LIKE 'AWAITING_DESIGN%'
                 OR s.to_state = 'AWAITING_HUMAN_APPROVAL' OR s.to_state LIKE 'PLAN\_%' THEN 'design'
            ELSE 'build'
        END AS phase,
        EXTRACT(EPOCH FROM COALESCE(s.next_at, b.ended_at) - s.at) AS secs) d
    GROUP BY s.session_id
)
SELECT p.context_id AS "context_id!: ContextId", p.session_id AS "session_id!", p.skill AS "skill!",
       p.marketplace_id, v.content_hash AS kit_hash, v.version AS kit_version,
       p.started_at AS "started_at!", b.ended_at AS "ended_at!",
       EXTRACT(EPOCH FROM b.ended_at - p.started_at)::float8 AS "wall_s!",
       COALESCE(pw.wait_s, 0)::float8 AS "prompt_wait_s!",
       COALESCE(qw.wait_s, 0)::float8 AS "question_wait_s!",
       COALESCE(pw.prompts, 0)::bigint AS "prompts!",
       COALESCE(qw.questions, 0)::bigint AS "questions!",
       p.turns AS "turns!", p.tool_calls AS "tool_calls!", p.tool_failures AS "tool_failures!",
       p.errors AS "errors!", p.rejected AS "rejected!", p.cost_microdollars AS "cost_microdollars!",
       p.contexts AS "contexts!",
       a.outcome, a.completion,
       ph.terminal_state, COALESCE(ph.transitions, 0)::bigint AS "transitions!",
       ph.requirements_s, ph.design_s, ph.build_s, ph.verify_s, ph.ship_s
FROM picked p
JOIN bounds b ON b.session_id = p.session_id
LEFT JOIN prompt_waits pw ON pw.session_id = p.session_id
LEFT JOIN question_waits qw ON qw.session_id = p.session_id
LEFT JOIN phases ph ON ph.session_id = p.session_id
LEFT JOIN conversation_analyses a ON a.context_id = p.context_id
LEFT JOIN LATERAL (SELECT mv.content_hash, mv.manifest->>'version' AS version
                   FROM marketplace_versions mv
                   WHERE mv.marketplace_id = p.marketplace_id
                     AND mv.content_hash = marketplace_version_at(p.marketplace_id, p.started_at)) v ON true
ORDER BY p.started_at DESC, p.session_id
