INSERT INTO context_notifications (context_id, agent_id, notification_type, notification_data, received_at, broadcasted)
VALUES ($1, $2, $3, $4::jsonb, $5, FALSE)
