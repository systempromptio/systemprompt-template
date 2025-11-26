UPDATE scheduled_jobs
SET last_run = $1,
    last_status = $2,
    last_error = $3,
    next_run = $4::timestamp,
    updated_at = $5
WHERE job_name = $6
