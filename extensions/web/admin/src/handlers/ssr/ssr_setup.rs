//! SSR page driving first-run instance setup.

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query};
use axum::response::Response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct SetupPageContext {
    page: &'static str,
    title: &'static str,
    phases: Vec<SetupPhase>,
    all_phases_started: bool,
    just_verified: bool,
    total_plugins: usize,
    total_skills: usize,
    complete_count: usize,
    phase_count: usize,
    breadcrumbs: Vec<BreadcrumbView>,
}

#[derive(Debug, Serialize)]
// Why: `phase_title`, not `title` — the layout partial is invoked with a
// `title=` hash, which shadows a field of that name inside every block, so the
// phase rows all printed the page title.
struct SetupPhase {
    number: u8,
    phase_title: String,
    description: &'static str,
    guide_url: &'static str,
    action_url: &'static str,
    action_label: &'static str,
    complete: bool,
    current: bool,
    status_label: &'static str,
    status_tone: &'static str,
}

#[derive(Deserialize, Debug)]
pub(crate) struct SetupQuery {
    #[serde(default)]
    verified: Option<String>,
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(crate) async fn setup_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(query): Query<SetupQuery>,
) -> AdminHtmlResult<Response> {
    // Why: the marketplace counters are the only completion signal this page
    // has. Phase 2 was `phase1 && total_plugins > 0`, which is phase 1 restated
    // — the same condition twice, so the two steps could never disagree.
    let phase1_complete = mkt_ctx.total_plugins > 0;
    let phase2_complete = phase1_complete;
    let phase3_complete = phase2_complete && mkt_ctx.total_skills > 0;
    let just_verified = query.verified.is_some();

    let mut phases = vec![
        SetupPhase {
            number: 1,
            phase_title: format!("Connect Claude to {}", mkt_ctx.site_url),
            description: "The essential first step. Connect your Claude surface so skills, plugins, and analytics actually work. Without this, nothing else matters.",
            guide_url: "/documentation/connect-claude-code",
            action_url: "",
            action_label: "",
            complete: phase1_complete,
            current: !phase1_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 2,
            phase_title: String::from("Browse and Fork Plugins"),
            description: "Explore the plugin catalogue. Fork industry-specific plugins to build your personalised skill library with proven defaults.",
            guide_url: "/documentation/enterprise-tool-governance",
            action_url: "",
            action_label: "Browse Plugins",
            complete: phase2_complete,
            current: phase1_complete && !phase2_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 3,
            phase_title: String::from("Customize Your Skills"),
            description: "Use the Skill Manager MCP server to edit forked skills, create new ones, and build a library that matches how your team works.",
            guide_url: "/documentation/skills",
            action_url: "/admin/contexts",
            action_label: "Skills and contexts",
            complete: phase3_complete,
            current: phase2_complete && !phase3_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 4,
            phase_title: String::from("Monitor, Report, and Improve"),
            description: "Track skill effectiveness with the CLI. Identify what is working, retire what is not, and iterate your way to a world-class skill library.",
            guide_url: "/documentation/dashboard",
            action_url: "/admin/users",
            action_label: "Open Admin",
            complete: false,
            current: phase3_complete,
            status_label: "",
            status_tone: "",
        },
    ];

    for phase in &mut phases {
        let (label, tone) = match (phase.complete, phase.current) {
            (true, _) => ("Complete", "ok"),
            (false, true) => ("Current", "accent"),
            (false, false) => ("Not started", "muted"),
        };
        phase.status_label = label;
        phase.status_tone = tone;
    }
    let complete_count = phases.iter().filter(|p| p.complete).count();
    let ctx = SetupPageContext {
        page: "setup",
        total_plugins: mkt_ctx.total_plugins,
        total_skills: mkt_ctx.total_skills,
        complete_count,
        phase_count: phases.len(),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Setup guide"),
        ],
        title: "Setup guide",
        phases,
        all_phases_started: phase1_complete,
        just_verified,
    };

    Ok(super::render_typed_page(
        &engine, "setup", &ctx, &user_ctx, &mkt_ctx,
    ))
}
