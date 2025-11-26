SELECT
  DATE(timestamp) as date,
  COUNT(*) as total_logs,
  COUNT(CASE WHEN level = 'ERROR' THEN 1 END) as error_count,
  ROUND(COUNT(CASE WHEN level = 'ERROR' THEN 1 END)::NUMERIC * 100 / NULLIF(COUNT(*), 0), 2) as error_rate_percent
FROM logs
WHERE timestamp >= NOW() - INTERVAL '7 days'
GROUP BY DATE(timestamp)
ORDER BY date DESC
