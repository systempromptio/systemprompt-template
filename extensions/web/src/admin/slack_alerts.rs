use systemprompt::models::SecretsBootstrap;

const SLACK_MAX_LENGTH: usize = 39_000;

fn alert_channel() -> Option<String> {
    SecretsBootstrap::get()
        .ok()
        .and_then(|s| s.get("activity_report_slack_channel").cloned())
}

pub fn send_alert(message: String) {
    tokio::spawn(async move {
        let Some(channel_id) = alert_channel() else {
            return;
        };
        let msg = if message.len() > SLACK_MAX_LENGTH {
            format!("{}... (truncated)", &message[..SLACK_MAX_LENGTH - 20])
        } else {
            message
        };
        if let Err(e) = send_to_slack(&channel_id, &msg).await {
            tracing::warn!(error = %e, "Failed to send Slack user alert");
        }
    });
}

async fn send_to_slack(
    channel_id: &str,
    message: &str,
) -> Result<(), crate::error::MarketplaceError> {
    // TODO: Implement Slack integration for foodles (previously used systemprompt_slack crate)
    tracing::debug!(channel_id, message, "Slack alert skipped: integration not yet implemented");
    Ok(())
}
