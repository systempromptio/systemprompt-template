SELECT
  COUNT(*) as total_logs,
  COUNT(CASE WHEN level = 'ERROR' THEN 1 END) as error_count,
  COUNT(CASE WHEN level = 'WARN' THEN 1 END) as warn_count,
  COUNT(CASE WHEN level = 'INFO' THEN 1 END) as info_count,
  COUNT(DISTINCT module) as unique_modules,
  COUNT(DISTINCT user_id) as unique_users,
  MAX(timestamp) as last_log_time
FROM logs
