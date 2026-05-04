pub(super) use crate::repositories::gamification_grp::{
    calculate_streaks, calculate_user_xp as calculate_user_xp_inner, populate_daily_usage,
    update_user_rank, UserRankParams, UserXpResult, UserXpScoringWeights,
};

use sqlx::PgPool;

use super::{ERROR_XP, PROMPT_XP, SESSION_XP, SUBAGENT_XP, TOKEN_XP_PER_1K, TOOL_USE_XP};

pub(super) async fn calculate_user_xp(
    pool: &PgPool,
    uid: &str,
) -> Result<UserXpResult, super::GamificationError> {
    let weights = UserXpScoringWeights {
        session_xp: SESSION_XP,
        tool_use_xp: TOOL_USE_XP,
        error_xp: ERROR_XP,
        prompt_xp: PROMPT_XP,
        subagent_xp: SUBAGENT_XP,
        token_xp_per_1k: TOKEN_XP_PER_1K,
    };
    Ok(calculate_user_xp_inner(pool, uid, weights).await?)
}
