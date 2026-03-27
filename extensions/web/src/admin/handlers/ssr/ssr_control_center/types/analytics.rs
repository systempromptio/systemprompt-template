use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct HealthObj {
    pub score: i64,
    pub label: &'static str,
    pub color_class: &'static str,
    pub total_sessions_30d: i64,
    pub avg_quality: String,
    pub goal_rate: i64,
    pub top_suggestion: String,
    pub has_suggestion: bool,
}

#[derive(Serialize, Clone)]
pub struct AchievementProgress {
    pub id: &'static str,
    pub name: &'static str,
    pub current: i64,
    pub threshold: i64,
    pub remaining: i64,
    pub pct: i64,
}

#[derive(Serialize, Clone)]
pub struct MetricRow {
    pub label: &'static str,
    pub value: String,
    pub yesterday_delta: String,
    pub yesterday_arrow: String,
    pub yesterday_sentiment: String,
    pub week_delta: String,
    pub week_arrow: String,
    pub week_sentiment: String,
    pub fortnight_delta: String,
    pub fortnight_arrow: String,
    pub fortnight_sentiment: String,
    pub global_delta: String,
    pub global_arrow: String,
    pub global_sentiment: String,
}

#[derive(Serialize, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct InsightsFlags {
    pub has_patterns: bool,
    pub has_skill_gaps: bool,
    pub has_recommendation: bool,
    pub has_highlights: bool,
    pub has_trends: bool,
}

#[derive(Serialize, Clone)]
pub struct InsightsData {
    pub summary: String,
    pub patterns: String,
    pub skill_gaps: String,
    pub top_recommendation: String,
    pub highlights: String,
    pub trends: String,
    #[serde(flatten)]
    pub flags: InsightsFlags,
}

#[derive(Serialize, Clone)]
pub struct HistoryEntry {
    pub date: String,
    pub sessions: i32,
    pub quality: f32,
    pub apm: f32,
    pub errors: i64,
}

#[derive(Serialize, Clone)]
pub struct CategoryBreakdownEntry {
    pub category: String,
    pub label: &'static str,
    pub count: usize,
    pub pct: f64,
    pub bar_width: f64,
}

#[derive(Serialize, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct StarRating {
    pub star_1: bool,
    pub star_2: bool,
    pub star_3: bool,
    pub star_4: bool,
    pub star_5: bool,
}

#[derive(Serialize, Clone)]
pub struct SkillEffectivenessEntry {
    pub skill_name: String,
    pub total_uses: i64,
    pub sessions_used_in: i64,
    pub avg_effectiveness: String,
    pub scored_sessions: i64,
    pub goal_achievement_pct: String,
    pub has_score: bool,
    #[serde(flatten)]
    pub stars: StarRating,
}

#[derive(Serialize, Clone)]
pub struct EntityCounts {
    pub plugins: i64,
    pub skills: i64,
    pub agents: i64,
    pub mcp_servers: i64,
    pub hooks: i64,
}

#[derive(Serialize, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AnalysisFlags {
    pub has_outcomes: bool,
    pub has_goal_outcome_map: bool,
    pub has_efficiency_metrics: bool,
    pub has_best_practices: bool,
}

#[derive(Serialize, Clone)]
pub struct AnalysisEntry {
    pub session_id: String,
    pub title: String,
    pub description: String,
    pub goal_summary: String,
    pub outcomes: Vec<String>,
    pub tags: String,
    pub tags_list: Vec<String>,
    pub goal_achieved: String,
    pub quality_score: i16,
    pub quality_class: &'static str,
    pub outcome: String,
    pub error_analysis: Option<String>,
    pub skill_assessment: Option<String>,
    pub recommendations: Option<String>,
    pub category: String,
    pub goal_outcome_map: Option<serde_json::Value>,
    pub efficiency_metrics: Option<serde_json::Value>,
    pub best_practices_checklist: Option<serde_json::Value>,
    pub improvement_hints: Option<String>,
    pub corrections_count: i32,
    pub total_turns: Option<i32>,
    pub session_duration_minutes: Option<i32>,
    #[serde(flatten)]
    pub flags: AnalysisFlags,
}
