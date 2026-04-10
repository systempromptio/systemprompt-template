use sqlx::PgPool;

#[derive(Debug)]
pub struct AchievementContext {
    pub user_id: String,
    pub total_xp: i64,
    pub unique_skills: i32,
    pub unique_plugins: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub rank_level: i32,
    pub total_tokens: i64,
    pub prompt_count: i64,
    pub subagent_count: i64,
    pub models_used: i32,
}

pub async fn check_achievements(
    pool: &PgPool,
    ctx: &AchievementContext,
) -> Result<(), anyhow::Error> {
    let (session_count, tool_count, custom_skills_count, error_count) =
        fetch_achievement_counts(pool, &ctx.user_id).await?;

    let mut to_unlock = Vec::new();

    let usage_counts = UsageCounts {
        sessions: session_count,
        tools: tool_count,
        custom_skills: custom_skills_count,
        prompts: ctx.prompt_count,
        subagents: ctx.subagent_count,
        total_xp: ctx.total_xp,
    };
    check_first_steps(&mut to_unlock, &usage_counts);
    check_milestones(&mut to_unlock, &usage_counts);
    check_exploration(
        &mut to_unlock,
        ctx.unique_skills,
        ctx.unique_plugins,
        ctx.models_used,
    );
    check_creation(&mut to_unlock, custom_skills_count);
    check_streaks(&mut to_unlock, ctx.current_streak, ctx.longest_streak);
    check_ranks(&mut to_unlock, ctx.rank_level);
    check_special(&mut to_unlock, error_count);
    check_tokens(&mut to_unlock, ctx.total_tokens);
    check_time_based(pool, &ctx.user_id, &mut to_unlock).await?;

    insert_achievements(pool, &ctx.user_id, &to_unlock).await?;

    Ok(())
}

async fn fetch_achievement_counts(
    pool: &PgPool,
    user_id: &str,
) -> Result<(i64, i64, i64, i64), anyhow::Error> {
    let session_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_SessionStart'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let tool_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_PostToolUse'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let custom_skills_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM user_skills WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let error_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_PostToolUseFailure'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok((session_count, tool_count, custom_skills_count, error_count))
}

struct UsageCounts {
    sessions: i64,
    tools: i64,
    custom_skills: i64,
    prompts: i64,
    subagents: i64,
    total_xp: i64,
}

fn check_first_steps(to_unlock: &mut Vec<&'static str>, counts: &UsageCounts) {
    if counts.sessions >= 1 {
        to_unlock.push("first_spark");
    }
    if counts.tools >= 1 {
        to_unlock.push("first_tool");
    }
    if counts.custom_skills >= 1 {
        to_unlock.push("first_custom_skill");
    }
    if counts.prompts >= 1 {
        to_unlock.push("first_prompt");
    }
    if counts.subagents >= 1 {
        to_unlock.push("first_subagent");
    }
}

fn check_milestones(to_unlock: &mut Vec<&'static str>, counts: &UsageCounts) {
    if counts.sessions >= 5 {
        to_unlock.push("five_sessions");
    }
    if counts.sessions >= 25 {
        to_unlock.push("twenty_five_sessions");
    }
    if counts.sessions >= 100 {
        to_unlock.push("hundred_sessions");
    }
    if counts.tools >= 50 {
        to_unlock.push("fifty_tools");
    }
    if counts.tools >= 250 {
        to_unlock.push("two_fifty_tools");
    }
    if counts.tools >= 1000 {
        to_unlock.push("thousand_tools");
    }
    if counts.total_xp >= 1000 {
        to_unlock.push("xp_1000");
    }
    if counts.prompts >= 50 {
        to_unlock.push("fifty_prompts");
    }
    if counts.prompts >= 500 {
        to_unlock.push("five_hundred_prompts");
    }
    if counts.subagents >= 10 {
        to_unlock.push("ten_subagents");
    }
}

fn check_exploration(
    to_unlock: &mut Vec<&'static str>,
    unique_skills: i32,
    unique_plugins: i32,
    models_used: i32,
) {
    if unique_skills >= 3 {
        to_unlock.push("three_unique_skills");
    }
    if unique_skills >= 10 {
        to_unlock.push("ten_unique_skills");
    }
    if unique_skills >= 20 {
        to_unlock.push("twenty_unique_skills");
    }
    if unique_plugins >= 3 {
        to_unlock.push("three_plugins");
    }
    if models_used >= 2 {
        to_unlock.push("two_models");
    }
    if models_used >= 4 {
        to_unlock.push("four_models");
    }
}

fn check_creation(to_unlock: &mut Vec<&'static str>, custom_skills_count: i64) {
    if custom_skills_count >= 5 {
        to_unlock.push("five_custom_skills");
    }
    if custom_skills_count >= 10 {
        to_unlock.push("ten_custom_skills");
    }
}

fn check_streaks(to_unlock: &mut Vec<&'static str>, current_streak: i32, longest_streak: i32) {
    let max_streak = std::cmp::max(current_streak, longest_streak);
    if max_streak >= 3 {
        to_unlock.push("streak_3");
    }
    if max_streak >= 7 {
        to_unlock.push("streak_7");
    }
    if max_streak >= 14 {
        to_unlock.push("streak_14");
    }
    if max_streak >= 30 {
        to_unlock.push("streak_30");
    }
}

fn check_ranks(to_unlock: &mut Vec<&'static str>, rank_level: i32) {
    if rank_level >= 3 {
        to_unlock.push("rank_3");
    }
    if rank_level >= 5 {
        to_unlock.push("rank_5");
    }
    if rank_level >= 7 {
        to_unlock.push("rank_7");
    }
    if rank_level >= 10 {
        to_unlock.push("rank_10");
    }
}

fn check_special(to_unlock: &mut Vec<&'static str>, error_count: i64) {
    if error_count >= 10 {
        to_unlock.push("error_handler");
    }
}

fn check_tokens(to_unlock: &mut Vec<&'static str>, total_tokens: i64) {
    if total_tokens >= 10_000 {
        to_unlock.push("token_10k");
    }
    if total_tokens >= 100_000 {
        to_unlock.push("token_100k");
    }
    if total_tokens >= 1_000_000 {
        to_unlock.push("token_1m");
    }
}

async fn check_time_based(
    pool: &PgPool,
    user_id: &str,
    to_unlock: &mut Vec<&'static str>,
) -> Result<(), anyhow::Error> {
    let has_early: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(HOUR FROM created_at) < 7)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if has_early {
        to_unlock.push("early_bird");
    }

    let has_late: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(HOUR FROM created_at) >= 22)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if has_late {
        to_unlock.push("night_owl");
    }

    let has_weekend: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(DOW FROM created_at) IN (0, 6))",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if has_weekend {
        to_unlock.push("weekend_warrior");
    }

    Ok(())
}

async fn insert_achievements(
    pool: &PgPool,
    user_id: &str,
    to_unlock: &[&str],
) -> Result<(), anyhow::Error> {
    for achievement_id in to_unlock {
        sqlx::query(
            "INSERT INTO employee_achievements (id, user_id, achievement_id) VALUES (gen_random_uuid()::TEXT, $1, $2) ON CONFLICT (user_id, achievement_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(achievement_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
