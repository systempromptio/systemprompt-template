-- Default marketplace plans (free + admin). Insert-if-absent: an operator's
-- edits to limits or display names survive every boot. Corrections to rows
-- that already exist on established databases are migration territory.
INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'free', 'Free', 'Default free tier', 'none', 'free_default', 0, NULL, 0,
    '{"ingestion":{"events_per_day":1000000,"content_bytes_per_day":10737418240,"sessions_per_day":10000},"entities":{"max_skills":1000,"max_agents":1000,"max_plugins":1000,"max_mcp_servers":1000,"max_hooks":1000},"features":{"ai":{"session_analysis":true,"daily_summaries":true},"apm_metrics":true,"gamification":true},"api":{"requests_per_minute":6000}}'::jsonb
)
ON CONFLICT (name) DO NOTHING;

INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'admin', 'Admin', 'Administrator unlimited access', 'none', 'admin_role', 0, 'admin', 99,
    '{"ingestion":{"events_per_day":9223372036854775807,"content_bytes_per_day":9223372036854775807,"sessions_per_day":9223372036854775807},"entities":{"max_skills":9223372036854775807,"max_agents":9223372036854775807,"max_plugins":9223372036854775807,"max_mcp_servers":9223372036854775807,"max_hooks":9223372036854775807},"features":{"ai":{"session_analysis":true,"daily_summaries":true},"apm_metrics":true,"gamification":true},"api":{"requests_per_minute":9223372036854775807}}'::jsonb
)
ON CONFLICT (name) DO NOTHING;
