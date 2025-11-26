INSERT INTO logs (
    user_id,
    session_id,
    level,
    module,
    message,
    task_id,
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7)