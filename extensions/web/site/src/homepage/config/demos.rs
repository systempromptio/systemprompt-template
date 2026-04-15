use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DemosConfig {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub quick_start: Vec<QuickStartStep>,
    #[serde(default)]
    pub categories: Vec<DemoCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStartStep {
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoCategory {
    pub id: String,
    pub title: String,
    pub tagline: String,
    pub story: String,
    pub cost: String,
    pub steps: Vec<DemoStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub path: String,
    pub name: String,
    pub label: String,
    pub narrative: String,
    pub outcome: String,
    pub commands: Vec<String>,
}
