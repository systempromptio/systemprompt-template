use sqlx::PgPool;

use super::achievements::{check_achievements, AchievementContext};
use super::rank_for_xp;
use super::recalculate_helpers::{
    calculate_streaks, calculate_user_xp, populate_daily_usage, update_user_rank, UserRankParams,
};

pub async fn recalculate_all(pool: &PgPool) -> Result<u64, super::GamificationError> {
    #[derive(sqlx::FromRow)]
    struct UserRow {
        user_id: String,
    }

    populate_daily_usage(pool).await?;

    let users = sqlx::query_as::<_, UserRow>(
        "SELECT DISTINCT e.user_id FROM plugin_usage_events e INNER JOIN users u ON u.id = e.user_id",
    )
    .fetch_all(pool)
    .await?;

    let mut updated = 0u64;

    for user in &users {
        let uid = &user.user_id;
        let (
            total_xp,
            events_count,
            unique_skills,
            unique_plugins,
            total_tokens,
            prompt_count,
            subagent_count,
            models_used,
        ) = calculate_user_xp(pool, uid).await?;
        let (current_streak, longest_streak, last_active_date) =
            calculate_streaks(pool, uid).await?;
        let (rank_level, rank_name) = rank_for_xp(total_xp);

        update_user_rank(&UserRankParams {
            pool,
            uid,
            total_xp,
            rank_level,
            rank_name,
            events_count,
            unique_skills,
            unique_plugins,
            current_streak,
            longest_streak,
            last_active_date,
        })
        .await?;

        let ctx = AchievementContext {
            user_id: uid.clone(),
            total_xp,
            unique_skills,
            unique_plugins,
            current_streak,
            longest_streak,
            rank_level,
            total_tokens,
            prompt_count,
            subagent_count,
            models_used,
        };
        check_achievements(pool, &ctx).await?;

        updated += 1;
    }

    Ok(updated)
}
