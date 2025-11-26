-- ============================================================================
-- REFERRER ANALYTICS VIEWS - Traffic source and campaign attribution analysis
-- ============================================================================

-- Top traffic sources
DROP VIEW IF EXISTS v_top_referrer_sources CASCADE;
CREATE VIEW v_top_referrer_sources AS
SELECT
    referrer_source,
    COUNT(*) as session_count,
    COUNT(DISTINCT user_id) as unique_users,
    AVG(request_count) as avg_requests_per_session,
    AVG(duration_seconds) as avg_session_duration_seconds,
    SUM(total_ai_cost_cents) as total_cost_cents
FROM user_sessions
WHERE referrer_source IS NOT NULL
GROUP BY referrer_source
ORDER BY session_count DESC;

-- Landing page conversion rates
DROP VIEW IF EXISTS v_landing_page_conversion CASCADE;
CREATE VIEW v_landing_page_conversion AS
SELECT
    landing_page,
    COUNT(*) as total_sessions,
    SUM(CASE WHEN user_type = 'anon' THEN 1 ELSE 0 END) as anonymous_sessions,
    SUM(CASE WHEN user_type = 'registered' THEN 1 ELSE 0 END) as registered_sessions,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) as converted_sessions,
    CAST(SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS NUMERIC) / NULLIF(COUNT(*), 0) * 100 as conversion_rate_percent,
    AVG(request_count) as avg_engagement
FROM user_sessions
WHERE landing_page IS NOT NULL
GROUP BY landing_page
HAVING COUNT(*) >= 5  -- Only show landing pages with at least 5 sessions
ORDER BY conversion_rate_percent DESC NULLS LAST;

-- UTM campaign performance
DROP VIEW IF EXISTS v_utm_campaign_performance CASCADE;
CREATE VIEW v_utm_campaign_performance AS
SELECT
    utm_source,
    utm_medium,
    utm_campaign,
    COUNT(*) as session_count,
    COUNT(DISTINCT user_id) as unique_users,
    SUM(CASE WHEN user_type = 'registered' THEN 1 ELSE 0 END) as registered_users,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) as conversions,
    CAST(SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS NUMERIC) / NULLIF(COUNT(*), 0) * 100 as conversion_rate_percent,
    AVG(duration_seconds) as avg_session_duration_seconds,
    SUM(total_ai_cost_cents) as total_cost_cents,
    AVG(total_ai_cost_cents) as avg_cost_per_session_cents
FROM user_sessions
WHERE utm_source IS NOT NULL
GROUP BY utm_source, utm_medium, utm_campaign
ORDER BY session_count DESC;

-- Referrer to landing page flow
DROP VIEW IF EXISTS v_referrer_landing_flow CASCADE;
CREATE VIEW v_referrer_landing_flow AS
SELECT
    referrer_source,
    landing_page,
    COUNT(*) as session_count,
    AVG(request_count) as avg_requests,
    AVG(duration_seconds) as avg_duration_seconds,
    SUM(CASE WHEN user_type = 'registered' THEN 1 ELSE 0 END) as registered_users
FROM user_sessions
WHERE referrer_source IS NOT NULL
AND landing_page IS NOT NULL
GROUP BY referrer_source, landing_page
HAVING COUNT(*) >= 3
ORDER BY session_count DESC;

-- Traffic source quality metrics
DROP VIEW IF EXISTS v_traffic_source_quality CASCADE;
CREATE VIEW v_traffic_source_quality AS
SELECT
    referrer_source,
    COUNT(*) as sessions,
    AVG(duration_seconds) as avg_duration_seconds,
    AVG(request_count) as avg_requests,
    AVG(ai_request_count) as avg_ai_requests,
    CAST(SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS NUMERIC) / NULLIF(COUNT(*), 0) * 100 as conversion_rate_percent,
    AVG(success_rate) as avg_success_rate,
    -- Quality score: weighted combination of engagement metrics
    (
        (AVG(duration_seconds) / 60.0 * 0.3) +  -- Duration weight
        (AVG(request_count) * 0.3) +             -- Request count weight
        (CAST(SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS NUMERIC) / NULLIF(COUNT(*), 0) * 100 * 0.4)  -- Conversion weight
    ) as quality_score
FROM user_sessions
WHERE referrer_source IS NOT NULL
GROUP BY referrer_source
HAVING COUNT(*) >= 10
ORDER BY quality_score DESC NULLS LAST;
