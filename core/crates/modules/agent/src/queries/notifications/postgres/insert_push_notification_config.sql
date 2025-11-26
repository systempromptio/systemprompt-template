INSERT INTO task_push_notification_configs
(id, task_id, url, endpoint, token, headers, authentication, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7::jsonb, $8, $9)
