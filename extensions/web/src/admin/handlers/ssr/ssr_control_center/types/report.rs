use serde::Serialize;

use super::analytics::{
    CategoryBreakdownEntry, EntityCounts, HistoryEntry, InsightsData, MetricRow,
};

#[derive(Serialize, Clone, Debug)]
pub struct ReportData {
    pub has_data: bool,
    pub report_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streak: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<Vec<MetricRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Vec<MetricRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub productivity: Option<Vec<MetricRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insights: Option<InsightsData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_counts: Option<EntityCounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_breakdown: Option<Vec<CategoryBreakdownEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_category_breakdown: Option<bool>,
}
