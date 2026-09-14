SELECT
    COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
        AS "total!",
    COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
        AND r.status NOT IN ('completed', 'success', 'pending', 'streaming'))::BIGINT
        AS "errors!",
    COALESCE(SUM(r.cost_microdollars)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "cost!",
    COALESCE(SUM(r.tokens_used)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "tokens!",
    COALESCE(SUM(r.reasoning_tokens)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "reasoning_tokens!",
    COALESCE(SUM(r.output_tokens)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "output_tokens!",
    COUNT(DISTINCT r.user_id)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
        AS "active_users!",
    COUNT(DISTINCT r.user_id)
        FILTER (WHERE r.created_at >= NOW() - INTERVAL '7 days')::BIGINT
        AS "weekly_active_users!",
    COUNT(*) FILTER (WHERE r.created_at >= $5 AND r.created_at < $1)::BIGINT
        AS "prev_total!",
    COUNT(*) FILTER (WHERE r.created_at >= $5 AND r.created_at < $1
        AND r.status NOT IN ('completed', 'success', 'pending', 'streaming'))::BIGINT
        AS "prev_errors!",
    COALESCE(SUM(r.cost_microdollars)
        FILTER (WHERE r.created_at >= $5 AND r.created_at < $1), 0)::BIGINT
        AS "prev_cost!",
    COALESCE(SUM(r.tokens_used)
        FILTER (WHERE r.created_at >= $5 AND r.created_at < $1), 0)::BIGINT
        AS "prev_tokens!",
    COUNT(DISTINCT r.user_id)
        FILTER (WHERE r.created_at >= $5 AND r.created_at < $1)::BIGINT
        AS "prev_active_users!",
    COUNT(DISTINCT r.user_id)
        FILTER (WHERE r.created_at >= NOW() - INTERVAL '14 days'
                  AND r.created_at < NOW() - INTERVAL '7 days')::BIGINT
        AS "prev_weekly_active_users!"
FROM ai_requests r
WHERE NOT r.synthetic
  AND (r.created_at >= $5 OR r.created_at >= NOW() - INTERVAL '14 days')
  AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
  AND ($4::TEXT IS NULL OR r.user_id = $4)
