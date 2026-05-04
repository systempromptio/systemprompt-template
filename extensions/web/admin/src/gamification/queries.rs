pub use crate::repositories::gamification_grp::{
    get_achievement_stats, get_department_leaderboard, get_department_scores, get_leaderboard,
    get_leaderboard_averages, get_user_gamification, LeaderboardAverages,
};

use sqlx::PgPool;

use crate::types::UserGamificationProfile;

pub async fn find_user_gamification(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<UserGamificationProfile>, sqlx::Error> {
    get_user_gamification(pool, user_id).await
}
