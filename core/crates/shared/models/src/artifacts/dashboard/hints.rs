use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub struct DashboardHints {
    pub layout: LayoutMode,
    pub refreshable: bool,
    pub refresh_interval_seconds: Option<u32>,
    pub drill_down_enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    #[default]
    Vertical,
    Grid,
    Tabs,
}

impl DashboardHints {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn with_refreshable(mut self, refreshable: bool) -> Self {
        self.refreshable = refreshable;
        self
    }

    pub const fn with_refresh_interval(mut self, seconds: u32) -> Self {
        self.refresh_interval_seconds = Some(seconds);
        self
    }

    pub const fn with_drill_down(mut self, enabled: bool) -> Self {
        self.drill_down_enabled = enabled;
        self
    }

    pub const fn with_layout(mut self, layout: LayoutMode) -> Self {
        self.layout = layout;
        self
    }

    pub fn generate_schema(&self) -> JsonValue {
        let mut schema = json!({
            "layout": self.layout,
            "refreshable": self.refreshable,
            "drill_down_enabled": self.drill_down_enabled
        });

        if let Some(interval) = self.refresh_interval_seconds {
            schema["refresh_interval_seconds"] = json!(interval);
        }

        schema
    }
}
