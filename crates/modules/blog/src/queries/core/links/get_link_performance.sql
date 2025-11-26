SELECT
    cl.id,
    cl.short_code,
    cl.target_url,
    cl.campaign_id,
    cl.campaign_name,
    cl.source_page,
    cl.click_count,
    cl.unique_click_count,
    cl.conversion_count,
    COUNT(DISTINCT lc.session_id) as session_count,
    COUNT(DISTINCT lc.user_id) as user_count,
    COUNT(CASE WHEN lc.is_conversion THEN 1 END) as actual_conversions,
    ROUND(100.0 * COUNT(CASE WHEN lc.is_conversion THEN 1 END) / NULLIF(COUNT(*), 0), 2) as conversion_rate,
    MIN(lc.clicked_at) as first_click_at,
    MAX(lc.clicked_at) as last_click_at,
    cl.created_at,
    cl.updated_at
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
WHERE cl.id = $1
GROUP BY cl.id;
