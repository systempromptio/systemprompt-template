UPDATE campaign_links
SET
    click_count = click_count + 1,
    unique_click_count = CASE WHEN $2 THEN unique_click_count + 1 ELSE unique_click_count END,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $1;
