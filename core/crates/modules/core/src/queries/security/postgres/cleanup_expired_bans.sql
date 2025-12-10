DELETE FROM banned_ips
WHERE expires_at IS NOT NULL
  AND expires_at < CURRENT_TIMESTAMP
  AND is_permanent = FALSE
