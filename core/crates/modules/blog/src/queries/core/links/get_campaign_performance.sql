SELECT
    cl.campaign_id,
    cl.campaign_name,
    COUNT(DISTINCT cl.id) as link_count,
    SUM(cl.click_count) as total_clicks,
    SUM(cl.unique_click_count) as total_unique_clicks,
    SUM(cl.conversion_count) as total_conversions,
    COUNT(DISTINCT lc.session_id) as session_count,
    ROUND(100.0 * SUM(cl.conversion_count) / NULLIF(SUM(cl.unique_click_count), 0), 2) as conversion_rate,
    MIN(cl.created_at) as campaign_start,
    MAX(lc.clicked_at) as last_activity
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
WHERE cl.campaign_id = $1
GROUP BY cl.campaign_id, cl.campaign_name;
