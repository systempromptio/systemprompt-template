SELECT
    id, link_id, session_id, user_id, referrer_page, referrer_url,
    clicked_at, user_agent, ip_address, device_type, country,
    is_first_click, is_conversion, conversion_at,
    time_on_page_seconds, scroll_depth_percent
FROM link_clicks
WHERE link_id = $1
ORDER BY clicked_at DESC
LIMIT $2 OFFSET $3;
