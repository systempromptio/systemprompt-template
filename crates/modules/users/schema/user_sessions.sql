-- ============================================================================
-- USER SESSIONS - Session-level analytics and activity tracking
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_sessions (
    session_id TEXT PRIMARY KEY,
    user_id VARCHAR(255),  -- Nullable to support anonymous users
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_activity_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    duration_seconds INTEGER,
    user_type VARCHAR(255) DEFAULT 'registered' CHECK (user_type IN ('anon', 'registered')),
    converted_at TIMESTAMP,
    expires_at TIMESTAMP DEFAULT (CURRENT_TIMESTAMP + INTERVAL '7 days'),
    client_id VARCHAR(255) NOT NULL DEFAULT 'sp_web',
    client_type VARCHAR(255) NOT NULL DEFAULT 'firstparty' CHECK (client_type IN ('cimd', 'firstparty', 'thirdparty', 'system', 'unknown')),
    request_count INTEGER DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION DEFAULT 0,
    success_rate DOUBLE PRECISION DEFAULT 1.0,
    error_count INTEGER DEFAULT 0,
    task_count INTEGER DEFAULT 0,
    message_count INTEGER DEFAULT 0,
    ai_request_count INTEGER DEFAULT 0,
    total_tokens_used INTEGER DEFAULT 0,
    total_ai_cost_cents BIGINT DEFAULT 0,
    ip_address TEXT,
    user_agent TEXT,
    device_type VARCHAR(255),
    browser TEXT,
    os TEXT,
    country TEXT,
    region TEXT,
    city TEXT,
    preferred_locale TEXT, -- From Accept-Language
    referrer_source VARCHAR(255),
    referrer_url TEXT,
    landing_page TEXT,
    entry_url TEXT,
    utm_source VARCHAR(100),
    utm_medium VARCHAR(100),
    utm_campaign VARCHAR(100),
    endpoints_accessed TEXT DEFAULT '[]',
    fingerprint_hash TEXT,
    is_bot BOOLEAN NOT NULL DEFAULT false,
    is_scanner BOOLEAN DEFAULT false,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

COMMENT ON COLUMN user_sessions.total_ai_cost_cents IS 'AI cost in microdollars (millionths of a dollar). Divide by 1,000,000 to get USD. Column name is legacy (should be total_ai_cost_microdollars).';
COMMENT ON COLUMN user_sessions.is_bot IS 'Whether this session was created by a bot/crawler (search engines, AI scrapers, social media bots, etc.)';
COMMENT ON COLUMN user_sessions.is_scanner IS 'Whether this session exhibits scanner/attacker behavior (accessing .php, .env, admin paths, high velocity)';
-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON user_sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_last_activity ON user_sessions(last_activity_at);
CREATE INDEX IF NOT EXISTS idx_sessions_country ON user_sessions(country);
CREATE INDEX IF NOT EXISTS idx_sessions_device_type ON user_sessions(device_type);
CREATE INDEX IF NOT EXISTS idx_sessions_ai_usage ON user_sessions(ai_request_count);
CREATE INDEX IF NOT EXISTS idx_sessions_cost ON user_sessions(total_ai_cost_cents);
CREATE INDEX IF NOT EXISTS idx_sessions_fingerprint ON user_sessions(fingerprint_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_fingerprint_activity ON user_sessions(fingerprint_hash, last_activity_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_type ON user_sessions(user_type);
CREATE INDEX IF NOT EXISTS idx_user_sessions_converted ON user_sessions(converted_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_client_id ON user_sessions(client_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_client_type ON user_sessions(client_type);
CREATE INDEX IF NOT EXISTS idx_user_sessions_client_activity ON user_sessions(client_id, last_activity_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_client_cost ON user_sessions(client_id, total_ai_cost_cents);
CREATE INDEX IF NOT EXISTS idx_user_sessions_referrer_source ON user_sessions(referrer_source);
CREATE INDEX IF NOT EXISTS idx_user_sessions_utm_source ON user_sessions(utm_source);
CREATE INDEX IF NOT EXISTS idx_user_sessions_landing_page ON user_sessions(landing_page);
-- Bot detection indexes
CREATE INDEX IF NOT EXISTS idx_user_sessions_is_bot ON user_sessions(is_bot);
CREATE INDEX IF NOT EXISTS idx_user_sessions_human_activity ON user_sessions(is_bot, last_activity_at) WHERE is_bot = false;
CREATE INDEX IF NOT EXISTS idx_user_sessions_bot_activity ON user_sessions(is_bot, started_at) WHERE is_bot = true;
CREATE INDEX IF NOT EXISTS idx_user_sessions_human_sessions ON user_sessions(is_bot, started_at, user_id) WHERE is_bot = false;
-- Scanner detection indexes
CREATE INDEX IF NOT EXISTS idx_user_sessions_is_scanner ON user_sessions(is_scanner);
CREATE INDEX IF NOT EXISTS idx_user_sessions_clean_traffic ON user_sessions(started_at) WHERE is_bot = false AND is_scanner = false;
-- Analytics optimization indexes
CREATE INDEX IF NOT EXISTS idx_sessions_referrer ON user_sessions(referrer_source, started_at) WHERE is_bot = false;
CREATE INDEX IF NOT EXISTS idx_sessions_utm ON user_sessions(utm_source, utm_campaign, utm_medium, started_at) WHERE is_bot = false;
CREATE INDEX IF NOT EXISTS idx_sessions_landing ON user_sessions(landing_page, is_bot);
CREATE INDEX IF NOT EXISTS idx_sessions_entry ON user_sessions(entry_url, is_bot);
CREATE INDEX IF NOT EXISTS idx_sessions_engagement ON user_sessions(duration_seconds, request_count, is_bot);
CREATE INDEX IF NOT EXISTS idx_sessions_quality ON user_sessions(success_rate, error_count, is_bot);
CREATE INDEX IF NOT EXISTS idx_sessions_fingerprint_time ON user_sessions(fingerprint_hash, started_at) WHERE is_bot = false;
CREATE INDEX IF NOT EXISTS idx_sessions_user_time ON user_sessions(user_id, started_at) WHERE is_bot = false;
CREATE INDEX IF NOT EXISTS idx_sessions_bot_time ON user_sessions(is_bot, started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_started_bot ON user_sessions(started_at DESC, is_bot);
-- Duration calculation handled at application level
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
GROUP BY client_id, client_type;

-- Scanner activity monitoring view
DROP VIEW IF EXISTS v_scanner_activity CASCADE;
CREATE VIEW v_scanner_activity AS
SELECT
    DATE(started_at) as date,
    COUNT(*) as scanner_sessions,
    COUNT(DISTINCT ip_address) as unique_ips,
    SUM(request_count) as total_requests,
    ROUND(AVG(request_count), 2) as avg_requests_per_session,
    ROUND(AVG(
        CASE
            WHEN ended_at IS NOT NULL THEN duration_seconds
            ELSE EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))::INTEGER
        END
    ), 2) as avg_duration_seconds
FROM user_sessions
WHERE is_scanner = true
  AND started_at >= CURRENT_TIMESTAMP - INTERVAL '30 days'
GROUP BY DATE(started_at)
ORDER BY date DESC;

-- Clean traffic view (human + legitimate bot traffic)
DROP VIEW IF EXISTS v_clean_traffic CASCADE;
CREATE VIEW v_clean_traffic AS
SELECT * FROM user_sessions
WHERE is_bot = false
  AND is_scanner = false;

-- Security threat analysis view
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
ORDER BY s.started_at DESC;

-- Top referrer sources view
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
GROUP BY referrer_source
ORDER BY session_count DESC;

-- UTM campaign performance view
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
GROUP BY utm_source, utm_medium, utm_campaign
ORDER BY session_count DESC;