UPDATE scheduled_jobs
SET run_count = run_count + 1
WHERE job_name = $1
