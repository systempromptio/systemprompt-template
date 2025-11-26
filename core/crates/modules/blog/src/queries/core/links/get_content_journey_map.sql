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
    ROUND(100.0 * COUNT(CASE WHEN lc.is_conversion THEN 1 END) / NULLIF(COUNT(*), 0), 2) as conversion_rate
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
LEFT JOIN markdown_content mc_source ON cl.source_content_id = mc_source.id
LEFT JOIN markdown_content mc_target ON cl.target_url LIKE '%' || mc_target.slug || '%'
WHERE cl.source_content_id IS NOT NULL
    AND cl.destination_type = 'internal'
GROUP BY cl.source_content_id, mc_source.slug, mc_source.title, cl.target_url, mc_target.slug, mc_target.title
HAVING COUNT(lc.id) > 0
ORDER BY click_count DESC
LIMIT $1 OFFSET $2;
