DELETE FROM logs
WHERE timestamp < datetime('now', '-' || $1 || ' days')