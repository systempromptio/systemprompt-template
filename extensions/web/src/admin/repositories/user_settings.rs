use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsRow {
    pub user_id: String,
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
    pub user_id: &'a str,
    pub display_name: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub notify_daily_summary: bool,
    pub notify_achievements: bool,
    pub leaderboard_opt_in: bool,
    pub timezone: &'a str,
}

pub async fn find_user_settings(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<UserSettingsRow>, sqlx::Error> {
    sqlx::query_as!(
        UserSettingsRow,
        "SELECT user_id, display_name, avatar_url, notify_daily_summary, notify_achievements, leaderboard_opt_in, timezone, achievement_email_sent_date, daily_report_email_sent_date, created_at, updated_at
         FROM user_settings WHERE user_id = $1",
        user_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_user_settings(
    pool: &PgPool,
    input: &UpsertUserSettings<'_>,
) -> Result<UserSettingsRow, sqlx::Error> {
    sqlx::query_as!(
        UserSettingsRow,
        "INSERT INTO user_settings (user_id, display_name, avatar_url, notify_daily_summary, notify_achievements, leaderboard_opt_in, timezone, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
         ON CONFLICT (user_id) DO UPDATE SET
             display_name = $2,
             avatar_url = $3,
             notify_daily_summary = $4,
             notify_achievements = $5,
             leaderboard_opt_in = $6,
             timezone = $7,
             updated_at = NOW()
         RETURNING user_id, display_name, avatar_url, notify_daily_summary, notify_achievements, leaderboard_opt_in, timezone, achievement_email_sent_date, daily_report_email_sent_date, created_at, updated_at",
        input.user_id,
        input.display_name,
        input.avatar_url,
        input.notify_daily_summary,
        input.notify_achievements,
        input.leaderboard_opt_in,
        input.timezone,
    )
    .fetch_one(pool)
    .await
}
