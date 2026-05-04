use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::{DepartmentScore, LeaderboardEntry};

struct LeaderRow {
    user_id: UserId,
    display_name: Option<String>,
    rank_level: i32,
    rank_name: String,
    total_xp: i64,
    events_count: i64,
    current_streak: i32,
    longest_streak: i32,
    achievement_count: i64,
    last_active_date: Option<NaiveDate>,
}

impl From<LeaderRow> for LeaderboardEntry {
    fn from(r: LeaderRow) -> Self {
        Self {
            user_id: r.user_id,
            display_name: r.display_name,
            rank_level: r.rank_level,
            rank_name: r.rank_name,
            total_xp: r.total_xp,
            events_count: r.events_count,
            current_streak: r.current_streak,
            longest_streak: r.longest_streak,
            achievement_count: r.achievement_count,
            last_active_date: r.last_active_date,
            total_sessions: 0,
            total_prompts: 0,
            total_tool_uses: 0,
            total_subagents: 0,
            unique_skills_count: 0,
            total_days_active: 0,
            period_xp: 0,
        }
    }
}

pub async fn get_leaderboard(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    department: Option<&str>,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let rows = if let Some(dept) = department {
        sqlx::query_as!(
            LeaderRow,
            r#"
            SELECT
                r.user_id AS "user_id: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                r.rank_level,
                r.rank_name,
                r.total_xp::BIGINT AS "total_xp!",
                r.events_count,
                r.current_streak,
                r.longest_streak,
                COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS "achievement_count!",
                r.last_active_date
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            WHERE u.department = $3
            ORDER BY r.total_xp DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
            dept,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            LeaderRow,
            r#"
            SELECT
                r.user_id AS "user_id: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                r.rank_level,
                r.rank_name,
                r.total_xp::BIGINT AS "total_xp!",
                r.events_count,
                r.current_streak,
                r.longest_streak,
                COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS "achievement_count!",
                r.last_active_date
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            ORDER BY r.total_xp DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(LeaderboardEntry::from).collect())
}

pub async fn get_department_leaderboard(
    pool: &PgPool,
    dept: &str,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let rows = sqlx::query_as!(
        LeaderRow,
        r#"
        SELECT
            r.user_id AS "user_id: UserId",
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            r.rank_level,
            r.rank_name,
            r.total_xp::BIGINT AS "total_xp!",
            r.events_count,
            r.current_streak,
            r.longest_streak,
            COALESCE((SELECT COUNT(*) FROM employee_achievements WHERE user_id = r.user_id), 0)::BIGINT AS "achievement_count!",
            r.last_active_date
        FROM employee_ranks r
        JOIN users u ON r.user_id = u.id
        WHERE u.department = $1
        ORDER BY r.total_xp DESC
        "#,
        dept,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(LeaderboardEntry::from).collect())
}

struct DeptRow {
    department: Option<String>,
    total_xp: i64,
    avg_xp: f64,
    user_count: i64,
    top_user_name: Option<String>,
    top_user_xp: i64,
}

impl From<DeptRow> for DepartmentScore {
    fn from(r: DeptRow) -> Self {
        Self {
            department: r.department.unwrap_or_default(),
            total_xp: r.total_xp,
            avg_xp: r.avg_xp,
            user_count: r.user_count,
            top_user_name: r.top_user_name,
            top_user_xp: r.top_user_xp,
        }
    }
}

pub async fn get_department_scores(
    pool: &PgPool,
    department: Option<&str>,
) -> Result<Vec<DepartmentScore>, sqlx::Error> {
    let rows = if let Some(dept) = department {
        sqlx::query_as!(
            DeptRow,
            r#"
            WITH top AS (
                SELECT DISTINCT ON (u.department)
                    u.department,
                    COALESCE(u.display_name, u.full_name, u.name) AS top_user_name,
                    r.total_xp::BIGINT AS top_user_xp
                FROM employee_ranks r
                JOIN users u ON r.user_id = u.id
                WHERE u.department IS NOT NULL AND u.department != ''
                ORDER BY u.department, r.total_xp DESC
            )
            SELECT
                u.department AS "department",
                COALESCE(SUM(r.total_xp), 0)::BIGINT AS "total_xp!",
                COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS "avg_xp!",
                COUNT(DISTINCT r.user_id)::BIGINT AS "user_count!",
                MAX(top.top_user_name) AS "top_user_name",
                COALESCE(MAX(top.top_user_xp), 0)::BIGINT AS "top_user_xp!"
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            LEFT JOIN top ON top.department = u.department
            WHERE u.department IS NOT NULL AND u.department != ''
              AND u.department = $1
            GROUP BY u.department
            ORDER BY 2 DESC
            "#,
            dept,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            DeptRow,
            r#"
            WITH top AS (
                SELECT DISTINCT ON (u.department)
                    u.department,
                    COALESCE(u.display_name, u.full_name, u.name) AS top_user_name,
                    r.total_xp::BIGINT AS top_user_xp
                FROM employee_ranks r
                JOIN users u ON r.user_id = u.id
                WHERE u.department IS NOT NULL AND u.department != ''
                ORDER BY u.department, r.total_xp DESC
            )
            SELECT
                u.department AS "department",
                COALESCE(SUM(r.total_xp), 0)::BIGINT AS "total_xp!",
                COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS "avg_xp!",
                COUNT(DISTINCT r.user_id)::BIGINT AS "user_count!",
                MAX(top.top_user_name) AS "top_user_name",
                COALESCE(MAX(top.top_user_xp), 0)::BIGINT AS "top_user_xp!"
            FROM employee_ranks r
            JOIN users u ON r.user_id = u.id
            LEFT JOIN top ON top.department = u.department
            WHERE u.department IS NOT NULL AND u.department != ''
            GROUP BY u.department
            ORDER BY 2 DESC
            "#,
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(DepartmentScore::from).collect())
}

#[derive(Debug, Clone, serde::Serialize, Copy)]
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

pub async fn get_leaderboard_averages(pool: &PgPool) -> Result<LeaderboardAverages, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE(AVG(r.total_xp), 0)::FLOAT8 AS "avg_xp!",
            COALESCE(AVG(r.events_count), 0)::FLOAT8 AS "avg_events!",
            COALESCE(AVG(r.current_streak), 0)::FLOAT8 AS "avg_streak!",
            COALESCE(AVG((SELECT COUNT(*) FROM employee_achievements ea WHERE ea.user_id = r.user_id)), 0)::FLOAT8 AS "avg_achievements!",
            COUNT(*)::BIGINT AS "total_users!"
        FROM employee_ranks r"#,
    )
    .fetch_one(pool)
    .await?;

    Ok(LeaderboardAverages {
        avg_xp: row.avg_xp,
        avg_sessions: row.avg_events,
        avg_prompts: 0.0,
        avg_tool_uses: 0.0,
        avg_subagents: 0.0,
        avg_streak: row.avg_streak,
        avg_achievements: row.avg_achievements,
        avg_days_active: 0.0,
        total_users: row.total_users,
    })
}
