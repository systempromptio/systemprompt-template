SELECT id, job_name, schedule, enabled, last_run, next_run, last_status, last_error, run_count, created_at, updated_at
FROM scheduled_jobs
WHERE enabled = true
ORDER BY job_name
