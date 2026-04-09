use sqlx::PgPool;

use super::super::types::{DepartmentScore, LeaderboardEntry};

pub async fn get_leaderboard(
    pool: &PgPool,
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
        .fetch_all(pool)
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
        .fetch_all(pool)
        .await
    }
}

pub async fn get_department_leaderboard(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await
}

pub async fn get_department_scores(
    pool: &PgPool,
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
        .fetch_all(pool)
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
        .fetch_all(pool)
        .await
    }
}

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

pub async fn get_leaderboard_averages(
    pool: &PgPool,
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
    .fetch_one(pool)
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
