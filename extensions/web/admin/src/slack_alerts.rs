//! Fire-and-forget Slack alerting for operational events.
//!
//! Alerting is off unless both a channel and a bot token are configured, and
//! delivery runs on a detached task so an outage in Slack never blocks the
//! caller. Messages are truncated to stay inside Slack's payload limit.
//!
//! Callers decide *when* an event is worth telling a human about; this module
//! only decides whether it can be delivered. A recurring condition should
//! alert on its transition, not on every observation — see the soft-cap
//! crossing in [`crate::gateway_org_budget`].

use systemprompt::config::SecretsBootstrap;

const SLACK_MAX_LENGTH: usize = 39_000;

fn bot_token() -> Option<String> {
    // Why: a placeholder counts as absent. `secrets.json` ships with
    // REPLACE_WITH_* values so setup can show what needs filling in; treating
    // one as a real token would make every alert a 401 in the logs.
    SecretsBootstrap::get()
        .ok()
        .and_then(|s| s.get("slack_bot_token").cloned())
        .filter(|t| !t.is_empty() && !t.starts_with("REPLACE_WITH"))
}

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
        send_to_slack(&channel_id, &msg).await;
    });
}

async fn send_to_slack(channel_id: &str, message: &str) {
    let Some(token) = bot_token() else {
        tracing::debug!(channel_id, "Slack alert skipped: no bot token configured");
        return;
    };

    let response = reqwest::Client::new()
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&serde_json::json!({ "channel": channel_id, "text": message }))
        .send()
        .await;

    // Why: every failure only warns. Alerting is best-effort by construction —
    // this runs on a detached task off the gateway's request path, so there is
    // nobody left to return an error to, and a Slack outage must never look
    // like a budget-guard failure.
    match response {
        Err(e) => tracing::warn!(error = %e, channel_id, "Slack alert delivery failed"),
        Ok(resp) => {
            // Why: Slack answers 200 with `{"ok": false, "error": "..."}` for
            // application-level failures (bad channel, revoked token), so the
            // HTTP status alone would report success for a message nobody saw.
            match resp.json::<SlackResponse>().await {
                Ok(body) if body.ok => {},
                Ok(body) => tracing::warn!(
                    channel_id,
                    error = body.error.as_deref().unwrap_or("unknown"),
                    "Slack rejected the alert"
                ),
                Err(e) => tracing::warn!(error = %e, channel_id, "unreadable Slack response"),
            }
        },
    }
}

#[derive(serde::Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}
