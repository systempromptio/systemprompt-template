SELECT
    (SELECT COUNT(*) FROM services WHERE status = 'running') as active_services,
    (SELECT COUNT(*) FROM services) as total_services,
    (SELECT pg_database_size(current_database()) / 1024.0 / 1024.0) as db_size_mb,
    (SELECT COUNT(*)
     FROM logs
     WHERE level = 'error'
     AND timestamp >= NOW() - INTERVAL '1 hour') as recent_errors,
    (SELECT COUNT(*)
     FROM logs
     WHERE level = 'critical'
     AND timestamp >= NOW() - INTERVAL '1 hour') as recent_critical,
    (SELECT COUNT(*)
     FROM logs
     WHERE level = 'warn'
     AND timestamp >= NOW() - INTERVAL '1 hour') as recent_warnings,
    (SELECT jsonb_agg(
        jsonb_build_object(
            'name', name,
            'status', status,
            'port', port,
            'updated_at', updated_at
        )
    ) FROM services) as services_json;
