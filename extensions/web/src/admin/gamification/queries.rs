use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{
    AchievementInfo, DepartmentScore, LeaderboardEntry, UnlockedAchievement,
    UserGamificationProfile,
};
use super::{xp_to_next_rank, ACHIEVEMENTS};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LeaderboardAverages {
    pub avg_xp: f64,
    pub avg_sessions: f64,
    pub avg_prompts: f64,
    pub avg_tool_uses: f64,
    pub avg_subagents: f64,
    pub avg_streak: f64,
    pub avg_achievements: f64,
    pub avg_days_active: f64,
    pub total_users: i64,
}

pub async fn get_leaderboard(
    pool: &Arc<PgPool>,
    limit: i64,
    offset: i64,
    department: Option<&str>,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, LeaderboardEntry>(
            r"
            SELECT
                r.user_id,
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.department,
                r.rank_level,
                r.rank_name,
                r.total_xp::BIGINT AS total_xp,
                r.events_count,
                r.current_streak,
                r.longest_streak,
                COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS achievement_count,
                r.last_active_date
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            WHERE u.department = $3
            ORDER BY r.total_xp DESC
            LIMIT $1 OFFSET $2
            ",
        )
        .bind(limit)
        .bind(offset)
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, LeaderboardEntry>(
            r"
            SELECT
                r.user_id,
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.department,
                r.rank_level,
                r.rank_name,
                r.total_xp::BIGINT AS total_xp,
                r.events_count,
                r.current_streak,
                r.longest_streak,
                COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS achievement_count,
                r.last_active_date
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            ORDER BY r.total_xp DESC
            LIMIT $1 OFFSET $2
            ",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn get_department_leaderboard(
    pool: &Arc<PgPool>,
    dept: &str,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardEntry>(
        r"
        SELECT
            r.user_id,
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            r.rank_level,
            r.rank_name,
            r.total_xp::BIGINT AS total_xp,
            r.events_count,
            r.current_streak,
            r.longest_streak,
            COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS achievement_count,
            r.last_active_date
        FROM employee_ranks r
        JOIN users u ON r.user_id = u.id
        WHERE u.department = $1
        ORDER BY r.total_xp DESC
        ",
    )
    .bind(dept)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_user_gamification(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Option<UserGamificationProfile>, sqlx::Error> {
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

    let Some(row) = sqlx::query_as::<_, RankRow>(
        r"
        SELECT
            r.user_id,
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            r.rank_level,
            r.rank_name,
            r.total_xp::BIGINT AS total_xp,
            r.events_count,
            r.unique_skills_count,
            r.unique_plugins_count,
            r.current_streak,
            r.longest_streak
        FROM employee_ranks r
        JOIN users u ON r.user_id = u.id
        WHERE r.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await?
    else {
        return Ok(None);
    };

    let achievements = sqlx::query_as::<_, UnlockedAchievement>(
        "SELECT achievement_id, unlocked_at FROM employee_achievements WHERE user_id = $1 ORDER BY unlocked_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await?;

    let rank_position: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM employee_ranks WHERE total_xp > (SELECT COALESCE(total_xp, 0) FROM employee_ranks WHERE user_id = $1)",
    )
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await?
    .unwrap_or(0);

    let (xp_needed, next_name) = xp_to_next_rank(row.total_xp);

    Ok(Some(UserGamificationProfile {
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
    }))
}

pub async fn get_achievement_stats(
    pool: &Arc<PgPool>,
) -> Result<Vec<AchievementInfo>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct AchievementCount {
        achievement_id: String,
        count: i64,
    }

    let total_users: i64 =
        sqlx::query_scalar("SELECT COALESCE(COUNT(*), 0)::BIGINT FROM employee_ranks")
            .fetch_one(pool.as_ref())
            .await?;

    let counts = sqlx::query_as::<_, AchievementCount>(
        "SELECT achievement_id, COUNT(*)::BIGINT AS count FROM employee_achievements GROUP BY achievement_id",
    )
    .fetch_all(pool.as_ref())
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

pub async fn get_department_scores(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<DepartmentScore>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, DepartmentScore>(
            r"
            SELECT
                u.department,
                COALESCE(SUM(r.total_xp), 0)::BIGINT AS total_xp,
                COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS avg_xp,
                COUNT(DISTINCT r.user_id)::BIGINT AS user_count,
                (SELECT COALESCE(u2.display_name, u2.full_name, u2.name)
                 FROM employee_ranks r2
                 JOIN users u2 ON r2.user_id = u2.id
                 WHERE u2.department = u.department
                 ORDER BY r2.total_xp DESC LIMIT 1) AS top_user_name,
                COALESCE((SELECT r3.total_xp FROM employee_ranks r3
                 JOIN users u3 ON r3.user_id = u3.id
                 WHERE u3.department = u.department
                 ORDER BY r3.total_xp DESC LIMIT 1), 0)::BIGINT AS top_user_xp
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            WHERE u.department IS NOT NULL AND u.department != ''
              AND u.department = $1
            GROUP BY u.department
            ORDER BY total_xp DESC
            ",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, DepartmentScore>(
            r"
            SELECT
                u.department,
                COALESCE(SUM(r.total_xp), 0)::BIGINT AS total_xp,
                COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS avg_xp,
                COUNT(DISTINCT r.user_id)::BIGINT AS user_count,
                (SELECT COALESCE(u2.display_name, u2.full_name, u2.name)
                 FROM employee_ranks r2
                 JOIN users u2 ON r2.user_id = u2.id
                 WHERE u2.department = u.department
                 ORDER BY r2.total_xp DESC LIMIT 1) AS top_user_name,
                COALESCE((SELECT r3.total_xp FROM employee_ranks r3
                 JOIN users u3 ON r3.user_id = u3.id
                 WHERE u3.department = u.department
                 ORDER BY r3.total_xp DESC LIMIT 1), 0)::BIGINT AS top_user_xp
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            WHERE u.department IS NOT NULL AND u.department != ''
            GROUP BY u.department
            ORDER BY total_xp DESC
            ",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

/// Alias for `get_user_gamification` used by SSR/SSE handlers.
pub async fn find_user_gamification(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Option<UserGamificationProfile>, sqlx::Error> {
    get_user_gamification(pool, user_id).await
}

/// Returns aggregate averages across all leaderboard users.
pub async fn get_leaderboard_averages(
    pool: &Arc<PgPool>,
) -> Result<LeaderboardAverages, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct AvgRow {
        avg_xp: Option<f64>,
        avg_events: Option<f64>,
        avg_streak: Option<f64>,
        avg_achievements: Option<f64>,
        total_users: i64,
    }

    let row = sqlx::query_as::<_, AvgRow>(
        r"SELECT
            COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS avg_xp,
            COALESCE(AVG(r.events_count), 0)::FLOAT8 AS avg_events,
            COALESCE(AVG(r.current_streak), 0)::FLOAT8 AS avg_streak,
            COALESCE(AVG((SELECT COUNT(*) FROM employee_achievements ea WHERE ea.user_id = r.user_id)), 0)::FLOAT8 AS avg_achievements,
            COUNT(*)::BIGINT AS total_users
        FROM employee_ranks r",
    )
    .fetch_one(pool.as_ref())
    .await?;

    Ok(LeaderboardAverages {
        avg_xp: row.avg_xp.unwrap_or(0.0),
        avg_sessions: row.avg_events.unwrap_or(0.0),
        avg_prompts: 0.0,
        avg_tool_uses: 0.0,
        avg_subagents: 0.0,
        avg_streak: row.avg_streak.unwrap_or(0.0),
        avg_achievements: row.avg_achievements.unwrap_or(0.0),
        avg_days_active: 0.0,
        total_users: row.total_users,
    })
}
