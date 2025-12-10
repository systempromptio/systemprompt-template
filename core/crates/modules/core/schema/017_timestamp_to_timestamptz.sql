-- Migration: Convert ALL TIMESTAMP columns to TIMESTAMPTZ for proper timezone handling
-- This ensures DateTime<Utc> mapping works correctly with sqlx compile-time queries

-- ============================================================================
-- STEP 1: Drop ALL views and materialized views (order matters due to dependencies)
-- ============================================================================

DROP MATERIALIZED VIEW IF EXISTS mv_top_content CASCADE;
DROP VIEW IF EXISTS v_content_performance CASCADE;
DROP VIEW IF EXISTS content CASCADE;
DROP VIEW IF EXISTS content_tags CASCADE;
DROP VIEW IF EXISTS tags CASCADE;
DROP VIEW IF EXISTS evaluation_metrics_daily CASCADE;
DROP VIEW IF EXISTS image_generation_stats CASCADE;
DROP VIEW IF EXISTS v_active_anonymous_sessions CASCADE;
DROP VIEW IF EXISTS v_client_errors CASCADE;
DROP VIEW IF EXISTS v_conversion_funnel CASCADE;
DROP VIEW IF EXISTS v_daily_conversions CASCADE;
DROP VIEW IF EXISTS v_landing_page_conversion CASCADE;
DROP VIEW IF EXISTS v_log_analytics_by_client CASCADE;
DROP VIEW IF EXISTS v_preconversion_engagement CASCADE;
DROP VIEW IF EXISTS v_referrer_landing_flow CASCADE;
DROP VIEW IF EXISTS v_time_to_conversion CASCADE;
DROP VIEW IF EXISTS v_top_referrer_sources CASCADE;
DROP VIEW IF EXISTS v_traffic_source_quality CASCADE;
DROP VIEW IF EXISTS v_utm_campaign_performance CASCADE;

-- ============================================================================
-- STEP 2: Convert ALL remaining TIMESTAMP columns to TIMESTAMPTZ
-- ============================================================================

-- markdown_content
ALTER TABLE markdown_content ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';
ALTER TABLE markdown_content ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
ALTER TABLE markdown_content ALTER COLUMN published_at TYPE TIMESTAMPTZ USING published_at AT TIME ZONE 'UTC';

-- markdown_tags
ALTER TABLE markdown_tags ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';
ALTER TABLE markdown_tags ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

-- markdown_content_tags
ALTER TABLE markdown_content_tags ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

-- content_performance_metrics
ALTER TABLE content_performance_metrics ALTER COLUMN first_view_at TYPE TIMESTAMPTZ USING first_view_at AT TIME ZONE 'UTC';
ALTER TABLE content_performance_metrics ALTER COLUMN last_view_at TYPE TIMESTAMPTZ USING last_view_at AT TIME ZONE 'UTC';
ALTER TABLE content_performance_metrics ALTER COLUMN last_calculated_at TYPE TIMESTAMPTZ USING last_calculated_at AT TIME ZONE 'UTC';

-- conversation_evaluations
ALTER TABLE conversation_evaluations ALTER COLUMN analyzed_at TYPE TIMESTAMPTZ USING analyzed_at AT TIME ZONE 'UTC';

-- generated_images
ALTER TABLE generated_images ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';
ALTER TABLE generated_images ALTER COLUMN deleted_at TYPE TIMESTAMPTZ USING deleted_at AT TIME ZONE 'UTC';

-- ============================================================================
-- STEP 3: Recreate ALL views
-- ============================================================================

-- tags view (simple passthrough from markdown_tags)
CREATE VIEW tags AS
SELECT id, name, slug, created_at, updated_at
FROM markdown_tags;

-- content view (passthrough from markdown_content with tsv)
CREATE VIEW content AS
SELECT
    id, slug, title, description, body, author, published_at,
    keywords, kind, image, category_id, source_id, version_hash,
    created_at, updated_at, links, public,
    to_tsvector('english'::regconfig,
        COALESCE(title, '') || ' ' ||
        COALESCE(description, '') || ' ' ||
        COALESCE(body, '') || ' ' ||
        COALESCE(keywords, '')
    ) AS tsv
FROM markdown_content;

-- content_tags view
CREATE VIEW content_tags AS
SELECT content_id, tag_id, created_at
FROM markdown_content_tags;

-- v_content_performance view
CREATE VIEW v_content_performance AS
SELECT
    mc.id,
    mc.slug,
    mc.title,
    mc.published_at,
    mc.kind,
    COALESCE(cpm.total_views, 0) AS total_views,
    COALESCE(cpm.total_unique_visitors, 0) AS unique_visitors,
    cpm.first_view_at,
    cpm.last_view_at,
    cpm.last_calculated_at
FROM markdown_content mc
LEFT JOIN content_performance_metrics cpm ON mc.id = cpm.content_id;

-- mv_top_content materialized view
CREATE MATERIALIZED VIEW mv_top_content AS
SELECT
    mc.id,
    mc.slug,
    mc.title,
    mc.published_at,
    COALESCE(cpm.total_views, 0) AS total_views,
    COALESCE(cpm.total_unique_visitors, 0) AS total_unique_visitors,
    COALESCE(cpm.views_7d, 0) AS views_7d,
    COALESCE(cpm.views_30d, 0) AS views_30d,
    COALESCE(cpm.last_view_at, mc.published_at) AS last_view_at
FROM markdown_content mc
LEFT JOIN content_performance_metrics cpm ON mc.id = cpm.content_id
WHERE mc.published_at <= CURRENT_TIMESTAMP
ORDER BY COALESCE(cpm.total_views, 0) DESC;

-- evaluation_metrics_daily view
CREATE VIEW evaluation_metrics_daily AS
SELECT
    date(analyzed_at) AS date,
    COUNT(*) AS total_conversations,
    (SUM(CASE WHEN goal_achieved = 'yes' THEN 1 ELSE 0 END)::NUMERIC * 100.0) / NULLIF(COUNT(*), 0) AS goal_success_rate,
    AVG(user_satisfied) AS avg_user_satisfaction,
    AVG(overall_score) AS avg_overall_score,
    AVG(total_turns) AS avg_turns,
    AVG(conversation_duration_seconds) AS avg_duration_seconds,
    (SUM(CASE WHEN completion_status = 'completed' THEN 1 ELSE 0 END)::NUMERIC * 100.0) / NULLIF(COUNT(*), 0) AS completion_rate
FROM conversation_evaluations
GROUP BY date(analyzed_at)
ORDER BY date(analyzed_at) DESC;

-- image_generation_stats view
CREATE VIEW image_generation_stats AS
SELECT
    provider,
    model,
    resolution,
    aspect_ratio,
    COUNT(*) AS total_images,
    AVG(generation_time_ms) AS avg_generation_time_ms,
    SUM(file_size_bytes) AS total_storage_bytes,
    SUM(cost_estimate) AS total_cost,
    date(created_at) AS generation_date
FROM generated_images
WHERE deleted_at IS NULL
GROUP BY provider, model, resolution, aspect_ratio, date(created_at);

-- v_client_errors view
CREATE VIEW v_client_errors AS
SELECT
    client_id,
    COUNT(*) AS error_count,
    COUNT(DISTINCT session_id) AS affected_sessions,
    MAX(timestamp) AS last_error
FROM logs
WHERE level = 'ERROR' AND client_id IS NOT NULL
GROUP BY client_id
ORDER BY COUNT(*) DESC;

-- v_log_analytics_by_client view
CREATE VIEW v_log_analytics_by_client AS
SELECT
    client_id,
    level,
    module,
    COUNT(*) AS log_count,
    MIN(timestamp) AS first_seen,
    MAX(timestamp) AS last_seen
FROM logs
WHERE client_id IS NOT NULL
GROUP BY client_id, level, module
ORDER BY COUNT(*) DESC;

-- v_active_anonymous_sessions view
CREATE VIEW v_active_anonymous_sessions AS
SELECT
    session_id,
    user_id,
    started_at,
    last_activity_at,
    expires_at,
    request_count,
    client_id
FROM user_sessions
WHERE user_type = 'anon'
  AND expires_at > CURRENT_TIMESTAMP
  AND ended_at IS NULL;

-- v_daily_conversions view
CREATE VIEW v_daily_conversions AS
SELECT
    date(started_at) AS date,
    COUNT(CASE WHEN user_type = 'anon' THEN 1 END) AS anonymous_sessions,
    COUNT(CASE WHEN user_type = 'registered' AND converted_at IS NOT NULL THEN 1 END) AS converted_sessions,
    ROUND(
        (COUNT(CASE WHEN user_type = 'registered' AND converted_at IS NOT NULL THEN 1 END)::NUMERIC * 100.0) /
        NULLIF(COUNT(CASE WHEN user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL) THEN 1 END), 0),
        2
    ) AS conversion_rate_pct
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY date(started_at)
ORDER BY date(started_at) DESC;

-- v_time_to_conversion view
CREATE VIEW v_time_to_conversion AS
SELECT
    session_id,
    user_id,
    started_at,
    converted_at,
    ROUND(EXTRACT(EPOCH FROM (converted_at - started_at)) / 60.0, 2) AS minutes_to_convert,
    CASE
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at)) / 60.0 < 5 THEN 'under_5_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at)) / 60.0 < 15 THEN '5_to_15_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at)) / 60.0 < 60 THEN '15_to_60_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at)) < 86400 THEN '1_to_24_hours'
        ELSE 'over_24_hours'
    END AS time_bucket
FROM user_sessions
WHERE user_type = 'registered' AND converted_at IS NOT NULL
ORDER BY converted_at DESC;

-- v_top_referrer_sources view
CREATE VIEW v_top_referrer_sources AS
SELECT
    referrer_source,
    COUNT(*) AS session_count,
    COUNT(DISTINCT user_id) AS unique_users,
    AVG(request_count) AS avg_requests_per_session,
    AVG(duration_seconds) AS avg_session_duration_seconds,
    SUM(total_ai_cost_cents) AS total_cost_cents
FROM user_sessions
WHERE referrer_source IS NOT NULL
GROUP BY referrer_source
ORDER BY COUNT(*) DESC;

-- v_landing_page_conversion view
CREATE VIEW v_landing_page_conversion AS
SELECT
    landing_page,
    COUNT(*) AS total_sessions,
    SUM(CASE WHEN user_type = 'anon' THEN 1 ELSE 0 END) AS anonymous_sessions,
    SUM(CASE WHEN user_type = 'registered' THEN 1 ELSE 0 END) AS registered_sessions,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS converted_sessions,
    (SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END)::NUMERIC / NULLIF(COUNT(*), 0)) * 100 AS conversion_rate_percent,
    AVG(request_count) AS avg_engagement
FROM user_sessions
WHERE landing_page IS NOT NULL
GROUP BY landing_page
HAVING COUNT(*) >= 5
ORDER BY conversion_rate_percent DESC NULLS LAST;

-- v_utm_campaign_performance view
CREATE VIEW v_utm_campaign_performance AS
SELECT
    utm_source,
    utm_medium,
    utm_campaign,
    COUNT(*) AS session_count,
    COUNT(DISTINCT user_id) AS unique_users,
    SUM(CASE WHEN user_type = 'registered' THEN 1 ELSE 0 END) AS registered_users,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS converted_users,
    (SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END)::NUMERIC / NULLIF(COUNT(*), 0)) * 100 AS conversion_rate,
    AVG(request_count) AS avg_engagement,
    SUM(total_ai_cost_cents) AS total_cost_cents
FROM user_sessions
WHERE utm_source IS NOT NULL OR utm_medium IS NOT NULL OR utm_campaign IS NOT NULL
GROUP BY utm_source, utm_medium, utm_campaign
ORDER BY session_count DESC;

-- v_conversion_funnel view
CREATE VIEW v_conversion_funnel AS
SELECT
    'total_visitors' AS stage,
    1 AS stage_order,
    COUNT(*) AS count
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
UNION ALL
SELECT
    'engaged_users' AS stage,
    2 AS stage_order,
    COUNT(*) AS count
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
  AND request_count > 1
UNION ALL
SELECT
    'registered_users' AS stage,
    3 AS stage_order,
    COUNT(*) AS count
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
  AND user_type = 'registered'
UNION ALL
SELECT
    'converted_users' AS stage,
    4 AS stage_order,
    COUNT(*) AS count
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
  AND converted_at IS NOT NULL
ORDER BY stage_order;

-- v_preconversion_engagement view
CREATE VIEW v_preconversion_engagement AS
SELECT
    session_id,
    user_id,
    started_at,
    converted_at,
    request_count,
    task_count,
    message_count,
    duration_seconds,
    landing_page,
    referrer_source
FROM user_sessions
WHERE user_type = 'registered'
  AND converted_at IS NOT NULL
ORDER BY converted_at DESC;

-- v_referrer_landing_flow view
CREATE VIEW v_referrer_landing_flow AS
SELECT
    referrer_source,
    landing_page,
    COUNT(*) AS session_count,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS conversions,
    (SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END)::NUMERIC / NULLIF(COUNT(*), 0)) * 100 AS conversion_rate
FROM user_sessions
WHERE referrer_source IS NOT NULL AND landing_page IS NOT NULL
GROUP BY referrer_source, landing_page
HAVING COUNT(*) >= 3
ORDER BY session_count DESC;

-- v_traffic_source_quality view
CREATE VIEW v_traffic_source_quality AS
SELECT
    referrer_source,
    COUNT(*) AS total_sessions,
    AVG(request_count) AS avg_requests,
    AVG(duration_seconds) AS avg_duration,
    (SUM(CASE WHEN request_count > 1 THEN 1 ELSE 0 END)::NUMERIC / NULLIF(COUNT(*), 0)) * 100 AS engagement_rate,
    (SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END)::NUMERIC / NULLIF(COUNT(*), 0)) * 100 AS conversion_rate
FROM user_sessions
WHERE referrer_source IS NOT NULL
GROUP BY referrer_source
HAVING COUNT(*) >= 5
ORDER BY conversion_rate DESC NULLS LAST;
