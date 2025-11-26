SELECT url, endpoint, token, headers, authentication
FROM task_push_notification_configs
WHERE task_id = $1 AND id = $2
