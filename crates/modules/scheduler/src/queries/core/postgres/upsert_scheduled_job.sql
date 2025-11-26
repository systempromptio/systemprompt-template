INSERT INTO scheduled_jobs (id, job_name, schedule, enabled, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT(job_name) DO UPDATE SET
    schedule = EXCLUDED.schedule,
    enabled = EXCLUDED.enabled,
    updated_at = EXCLUDED.updated_at
