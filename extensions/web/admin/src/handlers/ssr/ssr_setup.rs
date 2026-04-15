use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct SetupPhase {
    number: u8,
    title: String,
    description: &'static str,
    guide_url: &'static str,
    action_url: &'static str,
    action_label: &'static str,
    complete: bool,
    current: bool,
}

#[derive(Deserialize, Debug)]
pub struct SetupQuery {
    #[serde(default)]
    verified: Option<String>,
}

pub async fn setup_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(query): Query<SetupQuery>,
) -> Response {
    let phase1_complete = mkt_ctx.total_plugins > 0;
    let phase2_complete = phase1_complete && mkt_ctx.total_plugins > 0;
    let phase3_complete = phase2_complete && mkt_ctx.total_skills > 0;
    let just_verified = query.verified.is_some();

    let phases = vec![
        SetupPhase {
            number: 1,
            title: format!("Connect Claude to {}", mkt_ctx.site_url),
            description: "The essential first step. Connect your Claude surface so skills, plugins, and analytics actually work. Without this, nothing else matters.",
            guide_url: "/documentation/integration-claude-code",
            action_url: "",
            action_label: "",
            complete: phase1_complete,
            current: !phase1_complete,
        },
        SetupPhase {
            number: 2,
            title: String::from("Browse and Fork Plugins"),
            description: "Explore the plugin catalogue. Fork industry-specific plugins to build your personalised skill library with proven defaults.",
            guide_url: "/documentation/browse-plugins",
            action_url: "/admin/browse/plugins/",
            action_label: "Browse Plugins",
            complete: phase2_complete,
            current: phase1_complete && !phase2_complete,
        },
        SetupPhase {
            number: 3,
            title: String::from("Customize Your Skills"),
            description: "Use the Skill Manager MCP server to edit forked skills, create new ones, and build a library that matches how your team works.",
            guide_url: "/documentation/skills",
            action_url: "/admin/my/skills/",
            action_label: "My Skills",
            complete: phase3_complete,
            current: phase2_complete && !phase3_complete,
        },
        SetupPhase {
            number: 4,
            title: String::from("Monitor, Report, and Improve"),
            description: "Track skill effectiveness with analytics. Identify what is working, retire what is not, and iterate your way to a world-class skill library.",
            guide_url: "/documentation/dashboard",
            action_url: "/control-center",
            action_label: "Control Center",
            complete: false,
            current: phase3_complete,
        },
    ];

    let data = json!({
        "page": "setup",
        "title": "Setup Guide",
        "phases": phases,
        "all_phases_started": phase1_complete,
        "just_verified": just_verified,
    });

    super::render_page(&engine, "setup", &data, &user_ctx, &mkt_ctx)
}
