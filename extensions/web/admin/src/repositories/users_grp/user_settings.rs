use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub notify_daily_summary: bool,
    pub notify_achievements: bool,
    pub leaderboard_opt_in: bool,
    pub timezone: String,
    pub achievement_email_sent_date: Option<NaiveDate>,
    pub daily_report_email_sent_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpsertUserSettings<'a> {
    pub user_id: &'a UserId,
    pub display_name: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub notify_daily_summary: bool,
    pub notify_achievements: bool,
    pub leaderboard_opt_in: bool,
    pub timezone: &'a str,
}

pub async fn find_user_settings(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserSettingsRow>, sqlx::Error> {
    let id = user_id.as_str();
    sqlx::query_as!(
        UserSettingsRow,
        r#"SELECT
             user_id AS "user_id!: UserId",
             display_name,
             avatar_url,
             notify_daily_summary,
             notify_achievements,
             leaderboard_opt_in,
             timezone,
             achievement_email_sent_date,
             daily_report_email_sent_date,
             created_at,
             updated_at
           FROM user_settings WHERE user_id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}
