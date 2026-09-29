SELECT s.marketplace_id AS "marketplace_id!: MarketplaceId",s.marketplace_hash AS "marketplace_hash!",
 s.plugin_id AS "plugin_id!: PluginId",s.skill AS "skill!",
 sum(s.invocations)::bigint AS "invocations!",sum(s.failures)::bigint AS "failed_invocations!",
 count(DISTINCT s.user_id)::bigint AS "users!",count(*)::bigint AS "conversations!",
 sum(f.turn_count)::bigint AS "requests!",sum(f.error_count)::bigint AS "failed!",
 sum(f.input_tokens+f.output_tokens)::bigint AS "tokens!",
 sum(f.cost_microdollars-f.side_call_cost_microdollars)::bigint AS "cost!",
 percentile_cont(0.5) WITHIN GROUP (ORDER BY f.p50_latency_ms) FILTER (WHERE f.p50_latency_ms IS NOT NULL) AS "p50_ms?",
 percentile_cont(0.95) WITHIN GROUP (ORDER BY f.p95_latency_ms) FILTER (WHERE f.p95_latency_ms IS NOT NULL) AS "p95_ms?",
 count(*) FILTER (WHERE f.p50_latency_ms IS NOT NULL)::bigint AS "latency_measured!"
FROM conversation_skill_facts s
JOIN conversation_facts f ON f.context_id=s.context_id
WHERE s.marketplace_id=$3 AND s.marketplace_hash=ANY($4)
 AND s.first_invoked_at>=$1 AND s.first_invoked_at<$2
GROUP BY s.marketplace_id,s.marketplace_hash,s.plugin_id,s.skill
ORDER BY s.marketplace_hash,sum(s.invocations) DESC,s.plugin_id,s.skill
