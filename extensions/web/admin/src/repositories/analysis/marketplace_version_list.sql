WITH cur AS (
 SELECT marketplace_id,content_hash,source,source_hash,first_seen_at,plugin_count,skill_count,manifest->>'name' AS name
 FROM marketplace_versions WHERE effective_until IS NULL
), hist AS (
 SELECT marketplace_id,count(*) AS versions,min(first_seen_at) AS first_seen_at,max(first_seen_at) AS last_changed_at
 FROM marketplace_versions GROUP BY 1
), served AS (
 SELECT s.marketplace_id,s.context_id,s.user_id,sum(s.invocations) AS invocations,sum(s.failures) AS failures
 FROM conversation_skill_facts s
 WHERE s.marketplace_id IS NOT NULL AND s.first_invoked_at>=$1 AND s.first_invoked_at<$2
 GROUP BY 1,2,3
), keyed AS (
 SELECT k.marketplace_id,sum(k.invocations) AS invocations,sum(k.failures) AS failed_invocations,
  count(DISTINCT k.user_id) AS users,sum(f.turn_count) AS requests,sum(f.error_count) AS failed,
  sum(f.cost_microdollars-f.side_call_cost_microdollars) AS cost
 FROM served k JOIN conversation_facts f ON f.context_id=k.context_id
 GROUP BY 1
)
SELECT h.marketplace_id AS "marketplace_id: MarketplaceId",c.name AS "name?",c.content_hash AS "content_hash?",c.source AS "source?",
 c.source_hash AS "source_hash?",c.plugin_count AS "plugin_count?",c.skill_count AS "skill_count?",
 h.versions::bigint AS "versions!",h.first_seen_at AS "first_seen_at!",h.last_changed_at AS "last_changed_at!",
 coalesce(k.invocations,0)::bigint AS "invocations!",coalesce(k.failed_invocations,0)::bigint AS "failed_invocations!",
 coalesce(k.users,0)::bigint AS "users!",coalesce(k.requests,0)::bigint AS "requests!",coalesce(k.failed,0)::bigint AS "failed!",
 coalesce(k.cost,0)::bigint AS "cost!"
FROM hist h
LEFT JOIN cur c ON c.marketplace_id=h.marketplace_id
LEFT JOIN keyed k ON k.marketplace_id=h.marketplace_id
ORDER BY (c.content_hash IS NULL),k.invocations DESC NULLS LAST,h.marketplace_id
