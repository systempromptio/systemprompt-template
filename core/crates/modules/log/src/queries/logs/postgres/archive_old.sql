INSERT INTO logs_archive
SELECT * FROM logs
WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '30 days'
