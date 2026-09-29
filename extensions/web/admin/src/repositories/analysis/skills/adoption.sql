-- Marketplace adoption from the record: who installed (distinct consumers
-- holding a verified receipt for any skill the marketplace owns, by host —
-- a stock, so never windowed), who was active in the window (distinct people
-- who invoked one of its skills) and what that activity cost. Entitlement is
-- resolved from the access-control rules in Rust and joined there, which is
-- why the consumer ids travel with the count.
WITH owned AS (
    SELECT o.marketplace_id, replace(o.id, '-', '_') AS skill_key FROM service_owned_ids o WHERE o.kind = 'skill'
), receipts AS (
    SELECT w.marketplace_id, r.consumer_id, r.host, r.verified_at
    FROM owned w
    JOIN managed_resources res ON res.kind = 'skill' AND res.resource_key = w.skill_key
    JOIN managed_installation_receipts r ON r.resource_id = res.id AND r.fully_verified AND r.consumer_id IS NOT NULL
), installs AS (
    SELECT marketplace_id,
           COUNT(DISTINCT consumer_id)::bigint AS installed,
           ARRAY_AGG(DISTINCT consumer_id) AS installed_consumers,
           COUNT(DISTINCT consumer_id) FILTER (WHERE host = 'claude-code')::bigint AS installed_claude_code,
           COUNT(DISTINCT consumer_id) FILTER (WHERE host = 'opencode')::bigint AS installed_opencode,
           COUNT(DISTINCT consumer_id) FILTER (WHERE host NOT IN ('claude-code', 'opencode'))::bigint AS installed_other,
           MAX(verified_at) AS last_install_at
    FROM receipts GROUP BY marketplace_id
), activity AS (
    SELECT o.marketplace_id,
           COUNT(*)::bigint AS invocations,
           COUNT(DISTINCT e.user_id)::bigint AS active_users,
           COUNT(DISTINCT e.session_id)::bigint AS sessions,
           COUNT(DISTINCT e.skill)::bigint AS skills_used
    FROM analysis_skill_version_events e
    JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = e.plugin_id
    WHERE e.invoked_at >= $1 AND e.invoked_at < $2 AND e.skill IS NOT NULL
      AND ($3::text[] IS NULL OR e.user_id = ANY($3))
    GROUP BY o.marketplace_id
), spend AS (
    SELECT x.marketplace_id,
           COUNT(DISTINCT f.context_id)::bigint AS conversations,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(f.input_tokens + f.output_tokens), 0)::bigint AS tokens,
           AVG(a.completion)::float8 AS completion_avg
    FROM (SELECT DISTINCT o.marketplace_id, e.session_id
          FROM analysis_skill_version_events e
          JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = e.plugin_id
          WHERE e.invoked_at >= $1 AND e.invoked_at < $2 AND e.skill IS NOT NULL
            AND ($3::text[] IS NULL OR e.user_id = ANY($3))) x
    JOIN conversation_facts f ON f.client_session_id = x.session_id
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    GROUP BY x.marketplace_id
), marketplaces AS (
    SELECT o.id AS marketplace_id FROM service_owned_ids o WHERE o.kind = 'marketplace'
    UNION
    SELECT o.marketplace_id FROM service_owned_ids o WHERE o.marketplace_id IS NOT NULL
)
SELECT m.marketplace_id AS "marketplace_id!: MarketplaceId",
       (SELECT COUNT(*)::bigint FROM service_owned_ids o WHERE o.kind = 'skill' AND o.marketplace_id = m.marketplace_id) AS "skills!",
       (SELECT COUNT(*)::bigint FROM service_owned_ids o WHERE o.kind = 'plugin' AND o.marketplace_id = m.marketplace_id) AS "plugins!",
       COALESCE(i.installed, 0)::bigint AS "installed!",
       COALESCE(i.installed_consumers, '{}'::text[]) AS "installed_consumers!",
       COALESCE(i.installed_claude_code, 0)::bigint AS "installed_claude_code!",
       COALESCE(i.installed_opencode, 0)::bigint AS "installed_opencode!",
       COALESCE(i.installed_other, 0)::bigint AS "installed_other!",
       i.last_install_at,
       COALESCE(a.invocations, 0)::bigint AS "invocations!",
       COALESCE(a.active_users, 0)::bigint AS "active_users!",
       COALESCE(a.sessions, 0)::bigint AS "sessions!",
       COALESCE(a.skills_used, 0)::bigint AS "skills_used!",
       COALESCE(s.conversations, 0)::bigint AS "conversations!",
       COALESCE(s.cost_microdollars, 0)::bigint AS "cost_microdollars!",
       COALESCE(s.tokens, 0)::bigint AS "tokens!",
       s.completion_avg,
       (SELECT v.content_hash FROM marketplace_versions v WHERE v.marketplace_id = m.marketplace_id AND v.effective_until IS NULL
         ORDER BY v.first_seen_at DESC LIMIT 1) AS current_hash,
       (SELECT COUNT(*)::bigint FROM marketplace_versions v WHERE v.marketplace_id = m.marketplace_id) AS "versions!"
FROM marketplaces m
LEFT JOIN installs i ON i.marketplace_id = m.marketplace_id
LEFT JOIN activity a ON a.marketplace_id = m.marketplace_id
LEFT JOIN spend s ON s.marketplace_id = m.marketplace_id
WHERE m.marketplace_id IS NOT NULL
ORDER BY COALESCE(a.invocations, 0) DESC, m.marketplace_id
