pub const RANKS: &[(i32, &str, i64)] = &[
    (1, "Spark", 0),
    (2, "Prompt Apprentice", 50),
    (3, "Token Tinkerer", 150),
    (4, "Context Crafter", 400),
    (5, "Neural Navigator", 800),
    (6, "Model Whisperer", 1500),
    (7, "Pipeline Architect", 3000),
    (8, "Singularity Sage", 5000),
    (9, "Emergent Mind", 8000),
    (10, "Superintelligence", 12000),
];

pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

pub const ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef {
        id: "first_spark",
        name: "First Spark",
        description: "Start your first AI session",
        category: "First Steps",
    },
    AchievementDef {
        id: "first_tool",
        name: "Tool Time",
        description: "Use your first AI skill",
        category: "First Steps",
    },
    AchievementDef {
        id: "first_custom_skill",
        name: "Skill Smith",
        description: "Create your first custom skill",
        category: "First Steps",
    },
    AchievementDef {
        id: "five_sessions",
        name: "Getting Warmed Up",
        description: "Complete 5 AI sessions",
        category: "Milestones",
    },
    AchievementDef {
        id: "twenty_five_sessions",
        name: "Quarter Century",
        description: "Complete 25 AI sessions",
        category: "Milestones",
    },
    AchievementDef {
        id: "hundred_sessions",
        name: "Centurion",
        description: "Complete 100 AI sessions",
        category: "Milestones",
    },
    AchievementDef {
        id: "fifty_tools",
        name: "Toolbox",
        description: "Use 50 tool actions",
        category: "Milestones",
    },
    AchievementDef {
        id: "two_fifty_tools",
        name: "Power User",
        description: "Use 250 tool actions",
        category: "Milestones",
    },
    AchievementDef {
        id: "thousand_tools",
        name: "Tool Titan",
        description: "Use 1000 tool actions",
        category: "Milestones",
    },
    AchievementDef {
        id: "three_unique_skills",
        name: "Skill Explorer",
        description: "Use 3 different skills",
        category: "Exploration",
    },
    AchievementDef {
        id: "ten_unique_skills",
        name: "Skill Collector",
        description: "Use 10 different skills",
        category: "Exploration",
    },
    AchievementDef {
        id: "twenty_unique_skills",
        name: "Skill Master",
        description: "Use 20 different skills",
        category: "Exploration",
    },
    AchievementDef {
        id: "three_plugins",
        name: "Plugin Pioneer",
        description: "Use 3 different plugins",
        category: "Exploration",
    },
    AchievementDef {
        id: "five_custom_skills",
        name: "Skill Artisan",
        description: "Create 5 custom skills",
        category: "Creation",
    },
    AchievementDef {
        id: "ten_custom_skills",
        name: "Skill Factory",
        description: "Create 10 custom skills",
        category: "Creation",
    },
    AchievementDef {
        id: "streak_3",
        name: "On a Roll",
        description: "Maintain a 3-day usage streak",
        category: "Streaks",
    },
    AchievementDef {
        id: "streak_7",
        name: "Week Warrior",
        description: "Maintain a 7-day usage streak",
        category: "Streaks",
    },
    AchievementDef {
        id: "streak_14",
        name: "Fortnight Force",
        description: "Maintain a 14-day usage streak",
        category: "Streaks",
    },
    AchievementDef {
        id: "streak_30",
        name: "Monthly Maven",
        description: "Maintain a 30-day usage streak",
        category: "Streaks",
    },
    AchievementDef {
        id: "rank_3",
        name: "Token Tinkerer",
        description: "Reach rank 3: Token Tinkerer",
        category: "Ranks",
    },
    AchievementDef {
        id: "rank_5",
        name: "Neural Navigator",
        description: "Reach rank 5: Neural Navigator",
        category: "Ranks",
    },
    AchievementDef {
        id: "rank_7",
        name: "Pipeline Architect",
        description: "Reach rank 7: Pipeline Architect",
        category: "Ranks",
    },
    AchievementDef {
        id: "rank_10",
        name: "Superintelligence",
        description: "Reach rank 10: Superintelligence",
        category: "Ranks",
    },
    AchievementDef {
        id: "early_bird",
        name: "Early Bird",
        description: "Use AI before 7am",
        category: "Special",
    },
    AchievementDef {
        id: "night_owl",
        name: "Night Owl",
        description: "Use AI after 10pm",
        category: "Special",
    },
    AchievementDef {
        id: "weekend_warrior",
        name: "Weekend Warrior",
        description: "Use AI on a weekend",
        category: "Special",
    },
    AchievementDef {
        id: "error_handler",
        name: "Error Handler",
        description: "Encounter and recover from 10 errors",
        category: "Special",
    },
    AchievementDef {
        id: "xp_1000",
        name: "Kilobyte Mind",
        description: "Earn 1000 total XP",
        category: "Milestones",
    },
    AchievementDef {
        id: "token_10k",
        name: "Token Spender",
        description: "Use 10,000 total tokens",
        category: "Tokens",
    },
    AchievementDef {
        id: "token_100k",
        name: "Token Whale",
        description: "Use 100,000 total tokens",
        category: "Tokens",
    },
    AchievementDef {
        id: "token_1m",
        name: "Megabyte Mind",
        description: "Use 1,000,000 total tokens",
        category: "Tokens",
    },
    AchievementDef {
        id: "first_prompt",
        name: "Hello World",
        description: "Submit your first prompt",
        category: "First Steps",
    },
    AchievementDef {
        id: "fifty_prompts",
        name: "Conversationalist",
        description: "Submit 50 prompts",
        category: "Milestones",
    },
    AchievementDef {
        id: "five_hundred_prompts",
        name: "Prompt Pro",
        description: "Submit 500 prompts",
        category: "Milestones",
    },
    AchievementDef {
        id: "first_subagent",
        name: "Delegation",
        description: "Spawn your first subagent",
        category: "First Steps",
    },
    AchievementDef {
        id: "ten_subagents",
        name: "Team Builder",
        description: "Spawn 10 subagents",
        category: "Milestones",
    },
    AchievementDef {
        id: "two_models",
        name: "Model Sampler",
        description: "Use 2 different AI models",
        category: "Exploration",
    },
    AchievementDef {
        id: "four_models",
        name: "Model Connoisseur",
        description: "Use 4 different AI models",
        category: "Exploration",
    },
];

pub const SESSION_XP: i32 = 5;
pub const TOOL_USE_XP: i32 = 10;
pub const FIRST_UNIQUE_SKILL_XP: i32 = 25;
pub const CUSTOM_SKILL_XP: i32 = 50;
pub const STREAK_BONUS_XP: i32 = 15;
pub const ERROR_XP: i32 = 2;
pub const PROMPT_XP: i32 = 3;
pub const SUBAGENT_XP: i32 = 15;
pub const TOKEN_XP_PER_1K: i32 = 1;

#[must_use]
pub fn rank_for_xp(xp: i64) -> (i32, &'static str) {
    let mut result = (1, "Spark");
    for &(level, name, threshold) in RANKS {
        if xp >= threshold {
            result = (level, name);
        } else {
            break;
        }
    }
    result
}

#[must_use]
pub fn xp_to_next_rank(xp: i64) -> (i64, Option<&'static str>) {
    for &(_, name, threshold) in RANKS {
        if threshold > xp {
            return (threshold - xp, Some(name));
        }
    }
    (0, None)
}
