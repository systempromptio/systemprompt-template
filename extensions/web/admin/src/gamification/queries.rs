use sqlx::PgPool;

use super::super::types::{AchievementInfo, UnlockedAchievement, UserGamificationProfile};
use super::{xp_to_next_rank, ACHIEVEMENTS};

pub use super::queries_leaderboard::{
    get_department_leaderboard, get_department_scores, get_leaderboard, get_leaderboard_averages,
    LeaderboardAverages,
};

#[derive(sqlx::FromRow)]
struct RankRow {
    user_id: String,
    display_name: Option<String>,
    rank_level: i32,
    rank_name: String,
    total_xp: i64,
    events_count: i64,
    unique_skills_count: i32,
    unique_plugins_count: i32,
    current_streak: i32,
    longest_streak: i32,
}

pub async fn get_user_gamification(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<UserGamificationProfile>, sqlx::Error> {
    let Some(row) = fetch_rank_row(pool, user_id).await? else {
        return Ok(None);
    };

    let achievements = sqlx::query_as!(
        UnlockedAchievement,
        "SELECT achievement_id, unlocked_at FROM employee_achievements WHERE user_id = $1 ORDER BY unlocked_at DESC",
        user_id,
    )
    .fetch_all(pool)
    .await?;

    let rank_position: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*)::BIGINT FROM employee_ranks WHERE total_xp > (SELECT COALESCE(total_xp, 0) FROM employee_ranks WHERE user_id = $1)",
        user_id,
    )
    .fetch_optional(pool)
    .await?
    .flatten()
    .unwrap_or(0);

    Ok(Some(build_gamification_profile(
        row,
        achievements,
        rank_position,
    )))
}

async fn fetch_rank_row(pool: &PgPool, user_id: &str) -> Result<Option<RankRow>, sqlx::Error> {
    sqlx::query_as!(
        RankRow,
        r#"
        SELECT
            r.user_id,
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            r.rank_level,
            r.rank_name,
            r.total_xp::BIGINT AS "total_xp!",
            r.events_count,
            r.unique_skills_count,
            r.unique_plugins_count,
            r.current_streak,
            r.longest_streak
        FROM employee_ranks r
        JOIN users u ON r.user_id = u.id
        WHERE r.user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
}

fn build_gamification_profile(
    row: RankRow,
    achievements: Vec<UnlockedAchievement>,
    rank_position: i64,
) -> UserGamificationProfile {
    let (xp_needed, next_name) = xp_to_next_rank(row.total_xp);

    UserGamificationProfile {
        user_id: row.user_id.into(),
        display_name: row.display_name,
        rank_level: row.rank_level,
        rank_name: row.rank_name,
        total_xp: row.total_xp,
        xp_to_next_rank: xp_needed,
        next_rank_name: next_name.map(String::from),
        events_count: row.events_count,
        unique_skills_count: row.unique_skills_count,
        unique_plugins_count: row.unique_plugins_count,
        current_streak: row.current_streak,
        longest_streak: row.longest_streak,
        achievements,
        rank_position: rank_position + 1,
    }
}

pub async fn get_achievement_stats(pool: &PgPool) -> Result<Vec<AchievementInfo>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct AchievementCount {
        achievement_id: String,
        count: i64,
    }

    let total_users: i64 =
        sqlx::query_scalar!("SELECT COALESCE(COUNT(*), 0)::BIGINT FROM employee_ranks")
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

    let counts = sqlx::query_as!(
        AchievementCount,
        r#"SELECT achievement_id, COUNT(*)::BIGINT AS "count!" FROM employee_achievements GROUP BY achievement_id"#,
    )
    .fetch_all(pool)
    .await?;

    let count_map: std::collections::HashMap<&str, i64> = counts
        .iter()
        .map(|c| (c.achievement_id.as_str(), c.count))
        .collect();

    let infos = ACHIEVEMENTS
        .iter()
        .map(|def| {
            let total_unlocked = count_map.get(def.id).copied().unwrap_or(0);
            let unlock_percentage = if total_users > 0 {
                (f64::from(i32::try_from(total_unlocked).unwrap_or(0))
                    / f64::from(i32::try_from(total_users).unwrap_or(1)))
                    * 100.0
            } else {
                0.0
            };
            AchievementInfo {
                id: def.id.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                category: def.category.to_string(),
                total_unlocked,
                unlock_percentage,
            }
        })
        .collect();

    Ok(infos)
}

pub async fn find_user_gamification(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<UserGamificationProfile>, sqlx::Error> {
    get_user_gamification(pool, user_id).await
}
