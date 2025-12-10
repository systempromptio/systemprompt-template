INSERT INTO banned_ips (
    ip_address,
    reason,
    expires_at,
    ban_count,
    last_offense_path,
    last_user_agent,
    is_permanent
)
VALUES ($1, $2, $3, 1, $4, $5, $6)
ON CONFLICT (ip_address) DO UPDATE SET
    ban_count = banned_ips.ban_count + 1,
    last_offense_path = $4,
    last_user_agent = $5,
    banned_at = CURRENT_TIMESTAMP,
    expires_at = CASE
        WHEN banned_ips.is_permanent THEN NULL
        ELSE $3
    END,
    is_permanent = CASE
        WHEN banned_ips.ban_count >= 3 THEN TRUE
        ELSE banned_ips.is_permanent
    END
