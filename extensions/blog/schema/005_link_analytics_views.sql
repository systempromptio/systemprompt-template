-- Blog extension: Link analytics views
-- Materialized analytics views for performance reporting

CREATE OR REPLACE VIEW v_link_performance AS
SELECT
    cl.id,
    cl.short_code,
    cl.target_url,
    cl.link_type,
    cl.campaign_id,
    cl.campaign_name,
    cl.source_content_id,
    cl.source_page,
    cl.link_text,
    cl.destination_type,
    cl.click_count,
    cl.unique_click_count,
    cl.conversion_count,
    cl.is_active,
    cl.created_at,
    cl.updated_at,
    COUNT(DISTINCT lc.session_id) as actual_session_count,
    COUNT(DISTINCT lc.user_id) as actual_user_count,
    COUNT(CASE WHEN lc.is_conversion THEN 1 END) as actual_conversions,
    ROUND(100.0 * COUNT(CASE WHEN lc.is_conversion THEN 1 END) / NULLIF(COUNT(DISTINCT lc.session_id), 0), 2) as conversion_rate,
    MIN(lc.clicked_at) as first_click_at,
    MAX(lc.clicked_at) as last_click_at,
    AVG(lc.time_on_page_seconds) as avg_time_on_page,
    AVG(lc.scroll_depth_percent) as avg_scroll_depth
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
GROUP BY cl.id, cl.short_code, cl.target_url, cl.link_type, cl.campaign_id,
         cl.campaign_name, cl.source_content_id, cl.source_page, cl.link_text,
         cl.destination_type, cl.click_count, cl.unique_click_count,
         cl.conversion_count, cl.is_active, cl.created_at, cl.updated_at;

CREATE OR REPLACE VIEW v_campaign_performance AS
SELECT
    cl.campaign_id,
    cl.campaign_name,
    COUNT(DISTINCT cl.id) as link_count,
    SUM(cl.click_count) as total_clicks,
    SUM(cl.unique_click_count) as total_unique_clicks,
    SUM(cl.conversion_count) as total_conversions,
    COUNT(DISTINCT lc.session_id) as total_sessions,
    COUNT(DISTINCT lc.user_id) as total_users,
    ROUND(100.0 * SUM(cl.conversion_count) / NULLIF(SUM(cl.unique_click_count), 0), 2) as conversion_rate,
    MIN(cl.created_at) as campaign_start,
    MAX(lc.clicked_at) as last_activity,
    COUNT(DISTINCT DATE(lc.clicked_at)) as active_days,
    ROUND(SUM(cl.click_count)::numeric / NULLIF(COUNT(DISTINCT DATE(lc.clicked_at)), 0), 2) as avg_clicks_per_day
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
WHERE cl.campaign_id IS NOT NULL
GROUP BY cl.campaign_id, cl.campaign_name;

CREATE OR REPLACE VIEW v_content_journey AS
SELECT
    cl.source_content_id,
    mc_source.slug as source_slug,
    mc_source.title as source_title,
    cl.target_url,
    mc_target.slug as target_slug,
    mc_target.title as target_title,
    COUNT(lc.id) as click_count,
    COUNT(DISTINCT lc.session_id) as unique_sessions,
    COUNT(CASE WHEN lc.is_conversion THEN 1 END) as conversions,
    ROUND(100.0 * COUNT(CASE WHEN lc.is_conversion THEN 1 END) / NULLIF(COUNT(*), 0), 2) as conversion_rate,
    ROUND(AVG(lc.time_on_page_seconds), 2) as avg_time_before_click,
    MIN(lc.clicked_at) as first_journey_at,
    MAX(lc.clicked_at) as last_journey_at
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
LEFT JOIN markdown_content mc_source ON cl.source_content_id = mc_source.id
LEFT JOIN markdown_content mc_target ON cl.target_url LIKE '%' || mc_target.slug || '%'
WHERE cl.source_content_id IS NOT NULL
    AND cl.destination_type = 'internal'
GROUP BY cl.source_content_id, mc_source.slug, mc_source.title,
         cl.target_url, mc_target.slug, mc_target.title
HAVING COUNT(lc.id) > 0;

CREATE OR REPLACE VIEW v_link_click_stream AS
SELECT
    lc.id as click_id,
    lc.clicked_at,
    cl.short_code,
    cl.campaign_name,
    cl.source_page,
    cl.target_url,
    lc.session_id,
    lc.user_id,
    lc.device_type,
    lc.country,
    lc.is_first_click,
    lc.is_conversion,
    lc.referrer_page
FROM link_clicks lc
JOIN campaign_links cl ON lc.link_id = cl.id
ORDER BY lc.clicked_at DESC;

CREATE OR REPLACE VIEW v_top_performing_links AS
SELECT
    cl.id,
    cl.short_code,
    cl.target_url,
    cl.campaign_name,
    cl.link_type,
    cl.click_count,
    cl.unique_click_count,
    cl.conversion_count,
    ROUND(100.0 * cl.conversion_count / NULLIF(cl.unique_click_count, 0), 2) as conversion_rate,
    COUNT(DISTINCT lc.session_id) as session_count,
    cl.created_at,
    EXTRACT(EPOCH FROM (NOW() - cl.created_at)) / 86400 as age_days,
    ROUND(cl.click_count::numeric / NULLIF(EXTRACT(EPOCH FROM (NOW() - cl.created_at)) / 86400, 0), 2) as clicks_per_day
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
WHERE cl.is_active = TRUE
GROUP BY cl.id, cl.short_code, cl.target_url, cl.campaign_name, cl.link_type,
         cl.click_count, cl.unique_click_count, cl.conversion_count, cl.created_at
HAVING cl.click_count > 0
ORDER BY cl.click_count DESC, cl.conversion_count DESC;

-- Additional indexes for analytics queries
CREATE INDEX IF NOT EXISTS idx_link_clicks_device_type ON link_clicks(device_type) WHERE device_type IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_link_clicks_country ON link_clicks(country) WHERE country IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_link_clicks_date ON link_clicks(CAST(clicked_at AT TIME ZONE 'UTC' AS date));
CREATE INDEX IF NOT EXISTS idx_link_clicks_referrer_page ON link_clicks(referrer_page) WHERE referrer_page IS NOT NULL;
