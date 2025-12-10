use chrono::Utc;
use serde_json::json;
use systemprompt_core_system::BroadcastEvent;

use super::A2aNotification;

pub async fn persist_notification(
    db: systemprompt_core_database::DbPool,
    context_id: &str,
    agent_id: &str,
    notification: &A2aNotification,
) -> Result<i32, anyhow::Error> {
    let pool = db.pool_arc()?;
    let notification_data =
        serde_json::to_value(notification).map_err(|e| anyhow::anyhow!("{}", e))?;

    let result = sqlx::query!(
        r#"INSERT INTO context_notifications (context_id, agent_id, notification_type, notification_data)
        VALUES ($1, $2, $3, $4)
        RETURNING id"#,
        context_id,
        agent_id,
        notification.method,
        notification_data
    )
    .fetch_one(pool.as_ref())
    .await?;

    Ok(result.id)
}

pub async fn process_notification(
    app_context: systemprompt_core_system::AppContext,
    notification: &A2aNotification,
) -> Result<(), anyhow::Error> {
    let db = app_context.db_pool();
    let pool = db.pool_arc()?;

    match notification.method.as_str() {
        "notifications/taskStatusUpdate" => {
            let task_id = notification
                .params
                .get("taskId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing taskId in notification"))?;

            let status = notification
                .params
                .get("status")
                .ok_or_else(|| anyhow::anyhow!("Missing status in notification"))?;

            let state = status
                .get("state")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing state in status"))?;

            let timestamp = status
                .get("timestamp")
                .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
                .unwrap_or_else(Utc::now);

            if state == "completed" {
                sqlx::query!(
                    r#"UPDATE agent_tasks SET
                        status = 'completed',
                        updated_at = $1,
                        completed_at = CURRENT_TIMESTAMP,
                        started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
                        execution_time_ms = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - COALESCE(started_at, CURRENT_TIMESTAMP))) * 1000
                    WHERE task_id = $2"#,
                    timestamp,
                    task_id
                )
                .execute(pool.as_ref())
                .await?;
            } else {
                sqlx::query!(
                    "UPDATE agent_tasks SET status = $1, updated_at = $2 WHERE task_id = $3",
                    state,
                    timestamp,
                    task_id
                )
                .execute(pool.as_ref())
                .await?;
            }

            Ok(())
        },
        "notifications/artifactCreated" => Ok(()),
        "notifications/messageAdded" => Ok(()),
        _ => Ok(()),
    }
}

pub async fn broadcast_notification(
    context_id: &str,
    user_id: &str,
    notification: &A2aNotification,
) -> Result<usize, anyhow::Error> {
    let mut total_broadcasts = 0;

    match notification.method.as_str() {
        "notifications/taskStatusUpdate" => {
            let status_timestamp = notification
                .params
                .get("status")
                .and_then(|s| s.get("timestamp"))
                .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
                .unwrap_or_else(Utc::now);

            let task_event = BroadcastEvent {
                event_type: "task_status_changed".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "task_id": notification.params.get("taskId"),
                    "status": notification.params.get("status"),
                    "task": notification.params.get("task"),
                }),
                timestamp: status_timestamp,
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, task_event)
                .await;
        },
        "notifications/artifactCreated" => {
            let artifact_event = BroadcastEvent {
                event_type: "artifact_created".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "artifact": notification.params.get("artifact"),
                    "task_id": notification.params.get("taskId"),
                }),
                timestamp: Utc::now(),
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, artifact_event)
                .await;
        },
        "notifications/messageAdded" => {
            let message_event = BroadcastEvent {
                event_type: "message_added".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "message_id": notification.params.get("messageId"),
                    "message": notification.params.get("message"),
                }),
                timestamp: Utc::now(),
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, message_event)
                .await;
        },
        _ => {},
    }

    Ok(total_broadcasts)
}

pub async fn mark_notification_broadcasted(
    db: systemprompt_core_database::DbPool,
    notification_id: i32,
) -> Result<(), anyhow::Error> {
    let pool = db.pool_arc()?;
    sqlx::query!(
        "UPDATE context_notifications SET broadcasted = true WHERE id = $1",
        notification_id
    )
    .execute(pool.as_ref())
    .await?;

    Ok(())
}
