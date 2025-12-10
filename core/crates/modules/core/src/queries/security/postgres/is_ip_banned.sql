SELECT ip_address, reason, banned_at, expires_at, is_permanent
FROM banned_ips
WHERE ip_address = $1
  AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
LIMIT 1
