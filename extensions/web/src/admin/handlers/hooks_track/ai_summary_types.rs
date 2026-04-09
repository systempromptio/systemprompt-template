use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoalOutcomeMapping {
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub outcome: String,
    #[serde(default)]
    pub achieved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct EfficiencyMetrics {
    #[serde(default)]
    pub total_turns: i32,
    #[serde(default)]
    pub duration_minutes: i32,
    #[serde(default)]
    pub corrections_count: i32,
    #[serde(default)]
    pub avg_turns_per_goal: f32,
    #[serde(default)]
    pub unnecessary_loops: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BestPracticeItem {
    #[serde(default)]
    pub practice: String,
    #[serde(default)]
    pub score: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAnalysis {
    pub title: String,
    pub description: String,
    #[serde(alias = "summary")]
    pub goal_summary: String,
    pub outcomes: Vec<String>,
    pub tags: Vec<String>,
    pub goal_achieved: String,
    pub quality_score: i16,
    pub outcome: String,
    #[serde(default)]
    pub error_analysis: Option<String>,
    #[serde(default)]
    pub skill_assessment: Option<String>,
    #[serde(default)]
    pub recommendations: Option<String>,
    #[serde(default)]
    pub skill_scores: Option<HashMap<String, i16>>,
    pub category: Option<String>,
    #[serde(default)]
    pub goal_outcome_map: Option<Vec<GoalOutcomeMapping>>,
    #[serde(default)]
    pub efficiency_metrics: Option<EfficiencyMetrics>,
    #[serde(default)]
    pub best_practices_checklist: Option<Vec<BestPracticeItem>>,
    #[serde(default)]
    pub improvement_hints: Option<String>,
    #[serde(default)]
    pub automation_ratio: Option<f32>,
    #[serde(default)]
    pub plan_mode_used: Option<bool>,
    #[serde(default)]
    pub client_surface: Option<String>,
}

impl SessionAnalysis {
    pub fn composed_summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.goal_summary.is_empty() {
            parts.push(self.goal_summary.clone());
        }
        if !self.outcomes.is_empty() {
            let bullets: Vec<String> = self.outcomes.iter().map(|o| format!("- {o}")).collect();
            parts.push(bullets.join("\n"));
        }
        parts.join("\n\n")
    }
}

pub fn session_analysis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string", "description": "What the user wanted to do, max 80 chars" },
            "description": { "type": "string", "description": "The user's primary goal in one sentence, max 200 chars" },
            "goal_summary": { "type": "string", "description": "What the user wanted to accomplish, 1-2 sentences, max 200 chars" },
            "category": {
                "type": "string",
                "enum": ["feature", "bugfix", "refactoring", "techdebt", "documentation", "discovery", "testing", "deployment", "configuration", "design", "review", "other"]
            },
            "goal_outcome_map": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "goal": { "type": "string" },
                        "outcome": { "type": "string" },
                        "achieved": { "type": "boolean" }
                    },
                    "required": ["goal", "outcome", "achieved"]
                }
            },
            "outcomes": { "type": "array", "items": { "type": "string" } },
            "tags": {
                "type": "array",
                "items": {
                    "type": "string",
                    "enum": ["coding", "research", "debugging", "shell", "exploration", "refactoring", "documentation", "deployment", "testing", "configuration", "design", "review"]
                }
            },
            "goal_achieved": { "type": "string", "enum": ["yes", "partial", "no"] },
            "quality_score": { "type": "integer", "minimum": 1, "maximum": 5 },
            "outcome": { "type": "string", "description": "End state in one sentence" },
            "efficiency_metrics": {
                "type": "object",
                "properties": {
                    "total_turns": { "type": "integer" },
                    "duration_minutes": { "type": "integer" },
                    "corrections_count": { "type": "integer" },
                    "avg_turns_per_goal": { "type": "number" },
                    "unnecessary_loops": { "type": "integer" }
                },
                "required": ["total_turns", "duration_minutes", "corrections_count", "avg_turns_per_goal", "unnecessary_loops"]
            },
            "best_practices_checklist": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "practice": { "type": "string" },
                        "score": { "type": "string", "enum": ["yes", "partial", "no", "n/a"] },
                        "note": { "type": "string" }
                    },
                    "required": ["practice", "score", "note"]
                }
            },
            "improvement_hints": { "type": "string" },
            "error_analysis": { "type": "string" },
            "skill_assessment": { "type": "string" },
            "skill_scores": { "type": "object" },
            "recommendations": { "type": "string" },
            "automation_ratio": { "type": "number" },
            "plan_mode_used": { "type": "boolean" },
            "client_surface": { "type": "string", "enum": ["cli", "vscode", "jetbrains", "desktop", "unknown"] }
        },
        "required": [
            "title", "description", "goal_summary", "category",
            "goal_outcome_map", "outcomes", "tags", "goal_achieved",
            "quality_score", "outcome", "efficiency_metrics",
            "best_practices_checklist"
        ]
    })
}

pub fn validate_analysis(mut analysis: SessionAnalysis) -> SessionAnalysis {
    analysis.quality_score = analysis.quality_score.clamp(1, 5);

    if !["yes", "partial", "no"].contains(&analysis.goal_achieved.as_str()) {
        analysis.goal_achieved = "unknown".to_string();
    }

    let allowed_tags = [
        "coding",
        "research",
        "debugging",
        "shell",
        "exploration",
        "refactoring",
        "documentation",
        "deployment",
        "testing",
        "configuration",
        "design",
        "review",
    ];
    analysis.tags.retain(|t| allowed_tags.contains(&t.as_str()));

    if let Some(ref mut scores) = analysis.skill_scores {
        for v in scores.values_mut() {
            *v = (*v).clamp(1, 5);
        }
    }

    let allowed_categories = [
        "feature",
        "bugfix",
        "refactoring",
        "techdebt",
        "documentation",
        "discovery",
        "testing",
        "deployment",
        "configuration",
        "design",
        "review",
        "other",
    ];
    if let Some(ref cat) = analysis.category {
        if !allowed_categories.contains(&cat.as_str()) {
            analysis.category = Some("other".to_string());
        }
    } else {
        analysis.category = Some("other".to_string());
    }

    if let Some(ref mut eff) = analysis.efficiency_metrics {
        eff.total_turns = eff.total_turns.max(0);
        eff.duration_minutes = eff.duration_minutes.max(0);
        eff.corrections_count = eff.corrections_count.max(0);
        eff.avg_turns_per_goal = eff.avg_turns_per_goal.max(0.0);
        eff.unnecessary_loops = eff.unnecessary_loops.max(0);
    }

    if let Some(ref mut checklist) = analysis.best_practices_checklist {
        for item in checklist.iter_mut() {
            if !["yes", "partial", "no", "n/a"].contains(&item.score.as_str()) {
                item.score = "n/a".to_string();
            }
        }
    }

    analysis
}
