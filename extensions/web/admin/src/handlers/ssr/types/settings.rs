//! Template context types for the settings page.

use serde::Serialize;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SettingsView {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SettingsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub settings: SettingsView,
    pub user_email: String,
    pub user_id: UserId,
    pub username: String,
    pub breadcrumbs: Vec<super::BreadcrumbView>,
}
