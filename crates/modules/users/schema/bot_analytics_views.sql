-- ============================================================================
-- BOT ANALYTICS VIEWS
-- ============================================================================
-- Created: 2025-11-10
-- Description: Analytics views specifically for bot traffic monitoring
--
-- These views provide insights into bot crawler activity separately from
-- human user analytics, enabling SEO monitoring and bot pattern analysis.
-- ============================================================================

-- ============================================================================
-- VIEW: Bot Traffic Summary
-- ============================================================================
-- Daily breakdown of bot sessions and requests
DROP VIEW IF EXISTS v_bot_traffic_summary CASCADE;
CREATE VIEW v_bot_traffic_summary AS
SELECT
    DATE(started_at) as date,
    COUNT(*) as bot_sessions,
    COUNT(DISTINCT user_agent) as unique_bots,
    SUM(request_count) as total_bot_requests,
    AVG(request_count) as avg_requests_per_session
FROM user_sessions
WHERE is_bot = true
GROUP BY DATE(started_at)
ORDER BY date DESC;

COMMENT ON VIEW v_bot_traffic_summary IS 'Daily summary of bot crawler activity';

-- ============================================================================
-- VIEW: Bot Type Breakdown
-- ============================================================================
-- Categorizes bot traffic by major bot types (search engines, AI scrapers, etc.)
DROP VIEW IF EXISTS v_bot_type_breakdown CASCADE;
CREATE VIEW v_bot_type_breakdown AS
SELECT
    CASE
        WHEN user_agent ILIKE '%googlebot%' THEN 'Google'
        WHEN user_agent ILIKE '%bingbot%' THEN 'Bing'
        WHEN user_agent ILIKE '%baiduspider%' THEN 'Baidu'
        WHEN user_agent ILIKE '%yandexbot%' THEN 'Yandex'
        WHEN user_agent ILIKE '%duckduckbot%' THEN 'DuckDuckGo'
        WHEN user_agent ILIKE '%meta-externalagent%' OR user_agent ILIKE '%facebookexternalhit%' THEN 'Facebook'
        WHEN user_agent ILIKE '%twitterbot%' THEN 'Twitter'
        WHEN user_agent ILIKE '%linkedinbot%' THEN 'LinkedIn'
        WHEN user_agent ILIKE '%chatgpt%' OR user_agent ILIKE '%gptbot%' THEN 'ChatGPT'
        WHEN user_agent ILIKE '%claude%' THEN 'Claude'
        WHEN user_agent ILIKE '%perplexity%' THEN 'Perplexity'
        WHEN user_agent ILIKE '%amazonbot%' THEN 'Amazon'
        WHEN user_agent ILIKE '%applebot%' THEN 'Apple'
        WHEN user_agent ILIKE '%semrushbot%' THEN 'Semrush'
        WHEN user_agent ILIKE '%ahrefsbot%' THEN 'Ahrefs'
        WHEN user_agent ILIKE '%petalbot%' THEN 'Huawei'
        ELSE 'Other'
    END as bot_type,
    COUNT(*) as session_count,
    ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER(), 1) as percentage,
    COUNT(DISTINCT ip_address) as unique_ips
FROM user_sessions
WHERE is_bot = true
GROUP BY bot_type
ORDER BY session_count DESC;

COMMENT ON VIEW v_bot_type_breakdown IS 'Bot traffic categorized by major bot types (search engines, AI scrapers, etc.)';

-- ============================================================================
-- VIEW: Traffic Composition (Human vs Bot)
-- ============================================================================
-- Compares human and bot traffic over time
DROP VIEW IF EXISTS v_traffic_composition CASCADE;
CREATE VIEW v_traffic_composition AS
SELECT
    DATE(started_at) as date,
    COUNT(CASE WHEN is_bot = false THEN 1 END) as human_sessions,
    COUNT(CASE WHEN is_bot = true THEN 1 END) as bot_sessions,
    COUNT(*) as total_sessions,
    ROUND(100.0 * COUNT(CASE WHEN is_bot = false THEN 1 END) / COUNT(*), 1) as human_pct,
    ROUND(100.0 * COUNT(CASE WHEN is_bot = true THEN 1 END) / COUNT(*), 1) as bot_pct
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE(started_at)
ORDER BY date DESC;

COMMENT ON VIEW v_traffic_composition IS 'Daily comparison of human vs bot traffic';

-- ============================================================================
-- VIEW: SEO Crawler Activity
-- ============================================================================
-- Focuses on search engine crawlers for SEO monitoring
DROP VIEW IF EXISTS v_seo_crawler_activity CASCADE;
CREATE VIEW v_seo_crawler_activity AS
SELECT
    DATE(started_at) as date,
    CASE
        WHEN user_agent ILIKE '%googlebot%' THEN 'Google'
        WHEN user_agent ILIKE '%bingbot%' THEN 'Bing'
        WHEN user_agent ILIKE '%baiduspider%' THEN 'Baidu'
        WHEN user_agent ILIKE '%yandexbot%' THEN 'Yandex'
        WHEN user_agent ILIKE '%duckduckbot%' THEN 'DuckDuckGo'
        ELSE 'Other'
    END as search_engine,
    COUNT(*) as crawl_sessions,
    SUM(request_count) as total_pages_crawled
FROM user_sessions
WHERE is_bot = true
  AND (
    user_agent ILIKE '%googlebot%' OR
    user_agent ILIKE '%bingbot%' OR
    user_agent ILIKE '%baiduspider%' OR
    user_agent ILIKE '%yandexbot%' OR
    user_agent ILIKE '%duckduckbot%'
  )
GROUP BY DATE(started_at), search_engine
ORDER BY date DESC, crawl_sessions DESC;

COMMENT ON VIEW v_seo_crawler_activity IS 'Search engine crawler activity for SEO monitoring';

-- ============================================================================
-- VIEW: AI Scraper Activity
-- ============================================================================
-- Tracks AI model scraping activity (ChatGPT, Claude, Perplexity, etc.)
DROP VIEW IF EXISTS v_ai_scraper_activity CASCADE;
CREATE VIEW v_ai_scraper_activity AS
SELECT
    DATE(started_at) as date,
    CASE
        WHEN user_agent ILIKE '%chatgpt%' OR user_agent ILIKE '%gptbot%' THEN 'ChatGPT'
        WHEN user_agent ILIKE '%claude%' THEN 'Claude'
        WHEN user_agent ILIKE '%perplexity%' THEN 'Perplexity'
        WHEN user_agent ILIKE '%anthropic%' THEN 'Anthropic'
        WHEN user_agent ILIKE '%cohere%' THEN 'Cohere'
        ELSE 'Other AI'
    END as ai_model,
    COUNT(*) as scrape_sessions,
    SUM(request_count) as total_requests
FROM user_sessions
WHERE is_bot = true
  AND (
    user_agent ILIKE '%chatgpt%' OR
    user_agent ILIKE '%gptbot%' OR
    user_agent ILIKE '%claude%' OR
    user_agent ILIKE '%perplexity%' OR
    user_agent ILIKE '%anthropic%' OR
    user_agent ILIKE '%cohere%'
  )
GROUP BY DATE(started_at), ai_model
ORDER BY date DESC, scrape_sessions DESC;

COMMENT ON VIEW v_ai_scraper_activity IS 'AI model scraping activity (ChatGPT, Claude, Perplexity, etc.)';

-- ============================================================================
-- VIEW: Bot vs Human Metrics Comparison
-- ============================================================================
-- Side-by-side comparison of key metrics for bot vs human traffic
DROP VIEW IF EXISTS v_bot_human_metrics_comparison CASCADE;
CREATE VIEW v_bot_human_metrics_comparison AS
SELECT
    'Human Traffic' as traffic_type,
    COUNT(*) as total_sessions,
    COUNT(DISTINCT user_id) as unique_users,
    SUM(request_count) as total_requests,
    AVG(request_count) as avg_requests_per_session,
    AVG(duration_seconds) as avg_session_duration_secs
FROM user_sessions
WHERE is_bot = false
  AND started_at >= CURRENT_DATE - INTERVAL '30 days'

UNION ALL

SELECT
    'Bot Traffic' as traffic_type,
    COUNT(*) as total_sessions,
    COUNT(DISTINCT user_id) as unique_users,
    SUM(request_count) as total_requests,
    AVG(request_count) as avg_requests_per_session,
    AVG(duration_seconds) as avg_session_duration_secs
FROM user_sessions
WHERE is_bot = true
  AND started_at >= CURRENT_DATE - INTERVAL '30 days';

COMMENT ON VIEW v_bot_human_metrics_comparison IS 'Side-by-side comparison of bot vs human traffic metrics';

-- ============================================================================
-- VIEW: Recent Bot Activity
-- ============================================================================
-- Shows most recent bot sessions for monitoring
DROP VIEW IF EXISTS v_recent_bot_activity CASCADE;
CREATE VIEW v_recent_bot_activity AS
SELECT
    session_id,
    started_at,
    user_agent,
    ip_address,
    country,
    request_count,
    CASE
        WHEN user_agent ILIKE '%googlebot%' THEN 'Google'
        WHEN user_agent ILIKE '%bingbot%' THEN 'Bing'
        WHEN user_agent ILIKE '%chatgpt%' THEN 'ChatGPT'
        WHEN user_agent ILIKE '%facebook%' THEN 'Facebook'
        WHEN user_agent ILIKE '%perplexity%' THEN 'Perplexity'
        ELSE 'Other'
    END as bot_type
FROM user_sessions
WHERE is_bot = true
ORDER BY started_at DESC
LIMIT 100;

COMMENT ON VIEW v_recent_bot_activity IS 'Most recent 100 bot sessions for monitoring';
