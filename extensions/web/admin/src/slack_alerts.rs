use systemprompt::config::SecretsBootstrap;

const SLACK_MAX_LENGTH: usize = 39_000;

fn alert_channel() -> Option<String> {
    // Why: secrets are loaded lazily; an empty/missing bootstrap is the
    // "Slack alerts disabled" state and must not log on every alert path.
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
        send_to_slack(&channel_id, &msg);
    });
}

fn send_to_slack(channel_id: &str, message: &str) {
    tracing::debug!(
        channel_id,
        message,
        "Slack alert skipped: integration not yet implemented"
    );
}
