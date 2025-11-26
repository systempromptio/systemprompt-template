-- ============================================================================
-- Migration: 001_add_soft_deletes.sql
-- Purpose: Add soft delete support to analytics and session tables
-- Reason: Prevent data loss from administrative operations while maintaining
--         audit trail and allowing data recovery
-- ============================================================================

-- Add deleted_at column to user_sessions table
ALTER TABLE IF EXISTS user_sessions
ADD COLUMN deleted_at TIMESTAMP DEFAULT NULL;

-- Add deleted_at column to analytics_events table
ALTER TABLE IF EXISTS analytics_events
ADD COLUMN deleted_at TIMESTAMP DEFAULT NULL;

-- Add deleted_at column to endpoint_requests table
ALTER TABLE IF EXISTS endpoint_requests
ADD COLUMN deleted_at TIMESTAMP DEFAULT NULL;

-- Create indexes for soft delete queries (WHERE deleted_at IS NULL)
-- These enable efficient filtering of "active" (non-deleted) records
CREATE INDEX IF NOT EXISTS idx_user_sessions_deleted_at
ON user_sessions(deleted_at)
WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_analytics_events_deleted_at
ON analytics_events(deleted_at)
WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_endpoint_requests_deleted_at
ON endpoint_requests(deleted_at)
WHERE deleted_at IS NOT NULL;

-- Compound indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_user_sessions_active_time
ON user_sessions(started_at DESC)
WHERE deleted_at IS NULL AND is_bot = FALSE;

CREATE INDEX IF NOT EXISTS idx_analytics_events_active_session
ON analytics_events(session_id, timestamp DESC)
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_endpoint_requests_active_session
ON endpoint_requests(session_id, requested_at DESC)
WHERE deleted_at IS NULL;

-- Update existing views to exclude soft-deleted records
-- This ensures all analytics queries automatically respect soft deletes

DROP VIEW IF EXISTS v_session_analytics_by_client CASCADE;
CREATE VIEW v_session_analytics_by_client AS
SELECT
    client_id,
    client_type,
    COUNT(*) as session_count,
    COUNT(DISTINCT user_id) as unique_users,
    SUM(request_count) as total_requests,
    AVG(duration_seconds) as avg_session_duration_seconds,
    AVG(avg_response_time_ms) as avg_response_time_ms,
    SUM(total_tokens_used) as total_tokens,
    SUM(total_ai_cost_cents) as total_cost_cents,
    MIN(started_at) as first_seen,
    MAX(last_activity_at) as last_seen
FROM user_sessions
WHERE client_type != 'system'
  AND is_bot = false
  AND deleted_at IS NULL
GROUP BY client_id, client_type
ORDER BY session_count DESC;

DROP VIEW IF EXISTS v_client_rate_limits CASCADE;
CREATE VIEW v_client_rate_limits AS
SELECT
    client_id,
    client_type,
    COUNT(*) as sessions_last_hour,
    MAX(started_at) as last_session_created
FROM user_sessions
WHERE started_at >= NOW() - INTERVAL '1 hour'
  AND is_bot = false
  AND deleted_at IS NULL
GROUP BY client_id, client_type;

DROP VIEW IF EXISTS v_client_conversion_rates CASCADE;
CREATE VIEW v_client_conversion_rates AS
SELECT
    client_id,
    client_type,
    COUNT(*) as total_sessions,
    SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) as converted_sessions,
    CAST(SUM(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) AS DOUBLE PRECISION) / COUNT(*) as conversion_rate
FROM user_sessions
WHERE user_type = 'anon'
  AND is_bot = false
  AND deleted_at IS NULL
GROUP BY client_id, client_type;

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
  AND is_bot = false
  AND deleted_at IS NULL
GROUP BY referrer_source
ORDER BY session_count DESC;

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
  AND is_bot = false
  AND deleted_at IS NULL
GROUP BY utm_source, utm_medium, utm_campaign
ORDER BY session_count DESC;

DROP VIEW IF EXISTS v_clean_traffic CASCADE;
CREATE VIEW v_clean_traffic AS
SELECT * FROM user_sessions
WHERE is_bot = false
  AND is_scanner = false
  AND deleted_at IS NULL;

DROP VIEW IF EXISTS v_security_threats CASCADE;
CREATE VIEW v_security_threats AS
SELECT
    s.session_id,
    s.ip_address,
    s.user_agent,
    s.country,
    s.started_at,
    s.request_count,
    EXTRACT(EPOCH FROM (COALESCE(s.ended_at, CURRENT_TIMESTAMP) - s.started_at))::INTEGER as duration_seconds,
    ROUND((s.request_count::numeric / NULLIF(EXTRACT(EPOCH FROM (COALESCE(s.ended_at, CURRENT_TIMESTAMP) - s.started_at))::numeric, 0) * 60), 2) as requests_per_minute,
    s.endpoints_accessed,
    CASE
        WHEN s.endpoints_accessed::text LIKE '%.env%' THEN 'credential_theft'
        WHEN s.endpoints_accessed::text LIKE '%.php%' THEN 'backdoor_scanning'
        WHEN s.endpoints_accessed::text LIKE '%admin%' THEN 'admin_bruteforce'
        WHEN s.user_agent ILIKE '%masscan%' OR s.user_agent ILIKE '%nmap%' THEN 'port_scanning'
        ELSE 'unknown_threat'
    END as threat_type
FROM user_sessions s
WHERE s.is_scanner = true
  AND s.started_at >= CURRENT_TIMESTAMP - INTERVAL '7 days'
  AND s.deleted_at IS NULL
ORDER BY s.started_at DESC;
