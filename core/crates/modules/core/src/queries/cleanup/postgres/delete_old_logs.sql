DELETE FROM logs
WHERE timestamp < NOW() - INTERVAL '7 days'
