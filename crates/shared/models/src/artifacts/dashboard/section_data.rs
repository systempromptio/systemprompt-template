use super::super::chart::ChartDataset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCardsData {
    pub cards: Vec<MetricCard>,
}

impl MetricsCardsData {
    pub const fn new(cards: Vec<MetricCard>) -> Self {
        Self { cards }
    }

    pub fn add_card(mut self, card: MetricCard) -> Self {
        self.cards.push(card);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCard {
    pub title: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MetricStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    Success,
    Warning,
    Error,
    Info,
}

impl MetricCard {
    pub fn new(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            subtitle: None,
            icon: None,
            status: None,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub const fn with_status(mut self, status: MetricStatus) -> Self {
        self.status = Some(status);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSectionData {
    pub chart_type: String,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

impl ChartSectionData {
    pub fn new(
        chart_type: impl Into<String>,
        labels: Vec<String>,
        datasets: Vec<ChartDataset>,
    ) -> Self {
        Self {
            chart_type: chart_type.into(),
            labels,
            datasets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSectionData {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sort: Option<SortConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    pub column: String,
    pub order: String,
}

impl TableSectionData {
    pub const fn new(columns: Vec<String>, rows: Vec<serde_json::Value>) -> Self {
        Self {
            columns,
            rows,
            sortable: None,
            default_sort: None,
        }
    }

    pub const fn with_sortable(mut self, sortable: bool) -> Self {
        self.sortable = Some(sortable);
        self
    }

    pub fn with_default_sort(
        mut self,
        column: impl Into<String>,
        order: impl Into<String>,
    ) -> Self {
        self.default_sort = Some(SortConfig {
            column: column.into(),
            order: order.into(),
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSectionData {
    pub services: Vec<ServiceStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_errors: Option<ErrorCounts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub size_mb: f64,
    pub status: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ErrorCounts {
    pub critical: i32,
    pub error: i32,
    pub warn: i32,
}

impl StatusSectionData {
    pub const fn new(services: Vec<ServiceStatus>) -> Self {
        Self {
            services,
            database: None,
            recent_errors: None,
        }
    }

    pub fn with_database(mut self, status: DatabaseStatus) -> Self {
        self.database = Some(status);
        self
    }

    pub const fn with_error_counts(mut self, counts: ErrorCounts) -> Self {
        self.recent_errors = Some(counts);
        self
    }
}

impl ServiceStatus {
    pub fn new(name: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: status.into(),
            uptime: None,
        }
    }

    pub fn with_uptime(mut self, uptime: impl Into<String>) -> Self {
        self.uptime = Some(uptime.into());
        self
    }
}

impl DatabaseStatus {
    pub fn new(size_mb: f64, status: impl Into<String>) -> Self {
        Self {
            size_mb,
            status: status.into(),
        }
    }
}

impl ErrorCounts {
    pub const fn new(critical: i32, error: i32, warn: i32) -> Self {
        Self {
            critical,
            error,
            warn,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSectionData {
    pub lists: Vec<ItemList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemList {
    pub title: String,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub rank: i32,
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
}

impl ListSectionData {
    pub const fn new(lists: Vec<ItemList>) -> Self {
        Self { lists }
    }
}

impl ItemList {
    pub fn new(title: impl Into<String>, items: Vec<ListItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

impl ListItem {
    pub fn new(rank: i32, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            rank,
            label: label.into(),
            value: value.into(),
            badge: None,
        }
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}
