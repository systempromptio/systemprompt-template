WITH served AS (
 SELECT s.marketplace_hash,s.context_id,s.user_id,sum(s.invocations) AS invocations,sum(s.failures) AS failures
 FROM conversation_skill_facts s
 WHERE s.marketplace_id=$3 AND s.first_invoked_at>=$1 AND s.first_invoked_at<$2
 GROUP BY 1,2,3
), keyed AS (
 SELECT k.marketplace_hash,sum(k.invocations) AS invocations,sum(k.failures) AS failed_invocations,
  count(DISTINCT k.user_id) AS users,count(*) AS conversations,
  sum(f.turn_count) AS requests,sum(f.error_count) AS failed,
  sum(f.input_tokens+f.output_tokens) AS tokens,
  sum(f.cost_microdollars-f.side_call_cost_microdollars) AS cost,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY f.p50_latency_ms) FILTER (WHERE f.p50_latency_ms IS NOT NULL) AS p50_ms,
  percentile_cont(0.95) WITHIN GROUP (ORDER BY f.p95_latency_ms) FILTER (WHERE f.p95_latency_ms IS NOT NULL) AS p95_ms,
  count(*) FILTER (WHERE f.p50_latency_ms IS NOT NULL) AS latency_measured
 FROM served k JOIN conversation_facts f ON f.context_id=k.context_id
 GROUP BY 1
)
SELECT v.marketplace_id AS "marketplace_id: MarketplaceId",v.content_hash,v.source,v.source_hash,v.origin,
 v.manifest AS "manifest: Json<MarketplaceManifest>",
 v.first_seen_at,v.last_seen_at,v.effective_until,v.plugin_count,v.skill_count,
 coalesce(k.invocations,0)::bigint AS "invocations!",coalesce(k.failed_invocations,0)::bigint AS "failed_invocations!",
 coalesce(k.users,0)::bigint AS "users!",coalesce(k.conversations,0)::bigint AS "conversations!",
 coalesce(k.requests,0)::bigint AS "requests!",coalesce(k.failed,0)::bigint AS "failed!",
 coalesce(k.tokens,0)::bigint AS "tokens!",coalesce(k.cost,0)::bigint AS "cost!",
 k.p50_ms AS "p50_ms?",k.p95_ms AS "p95_ms?",
 coalesce(k.latency_measured,0)::bigint AS "latency_measured!"
FROM marketplace_versions v
LEFT JOIN keyed k ON k.marketplace_hash=v.content_hash
WHERE v.marketplace_id=$3
ORDER BY (v.effective_until IS NULL) DESC,v.first_seen_at DESC
