SELECT COUNT(*) as click_count
FROM link_clicks
WHERE link_id = $1 AND session_id = $2;
