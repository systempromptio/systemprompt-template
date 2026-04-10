use std::path::Path;
use systemprompt::extension::AssetDefinition;

pub(super) fn page_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let pages = storage_js.join("pages");
    let mut v = page_admin_core_js(&pages);
    v.extend(page_admin_cc_js(&pages));
    v.extend(page_admin_marketplace_js(&pages));
    v.extend(page_admin_my_js(&pages));
    v.extend(page_admin_org_js(&pages));
    v.extend(page_admin_plugin_js(&pages));
    v
}

fn page_admin_core_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(pages.join("admin-access.js"), "js/pages/admin-access.js"),
        AssetDefinition::js(
            pages.join("admin-access-bulk.js"),
            "js/pages/admin-access-bulk.js",
        ),
        AssetDefinition::js(
            pages.join("admin-access-panel.js"),
            "js/pages/admin-access-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-achievements.js"),
            "js/pages/admin-achievements.js",
        ),
        AssetDefinition::js(
            pages.join("admin-agent-edit.js"),
            "js/pages/admin-agent-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-agents-helpers.js"),
            "js/pages/admin-agents-helpers.js",
        ),
        AssetDefinition::js(pages.join("admin-audit.js"), "js/pages/admin-audit.js"),
        AssetDefinition::js(pages.join("admin-billing.js"), "js/pages/admin-billing.js"),
        AssetDefinition::js(
            pages.join("admin-dashboard-charts.js"),
            "js/pages/admin-dashboard-charts.js",
        ),
        AssetDefinition::js(
            pages.join("admin-dashboard.js"),
            "js/pages/admin-dashboard.js",
        ),
        AssetDefinition::js(pages.join("admin-events.js"), "js/pages/admin-events.js"),
        AssetDefinition::js(pages.join("admin-export.js"), "js/pages/admin-export.js"),
        AssetDefinition::js(pages.join("admin-jobs.js"), "js/pages/admin-jobs.js"),
        AssetDefinition::js(
            pages.join("admin-leaderboard.js"),
            "js/pages/admin-leaderboard.js",
        ),
        AssetDefinition::js(pages.join("admin-profile.js"), "js/pages/admin-profile.js"),
        AssetDefinition::js(
            pages.join("admin-settings.js"),
            "js/pages/admin-settings.js",
        ),
        AssetDefinition::js(
            pages.join("admin-skill-edit.js"),
            "js/pages/admin-skill-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-users-actions.js"),
            "js/pages/admin-users-actions.js",
        ),
        AssetDefinition::js(pages.join("admin-users.js"), "js/pages/admin-users.js"),
    ]
}

fn page_admin_cc_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-cc-report-build.js"),
            "js/pages/admin-cc-report-build.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-actions.js"),
            "js/pages/admin-control-center-actions.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-panels.js"),
            "js/pages/admin-control-center-panels.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-report.js"),
            "js/pages/admin-control-center-report.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-sse.js"),
            "js/pages/admin-control-center-sse.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center.js"),
            "js/pages/admin-control-center.js",
        ),
    ]
}

fn page_admin_marketplace_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-browse-plugins.js"),
            "js/pages/admin-browse-plugins.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse-panel-helpers.js"),
            "js/pages/admin-marketplace-browse-panel-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse-panel.js"),
            "js/pages/admin-marketplace-browse-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse.js"),
            "js/pages/admin-marketplace-browse.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions-panel-helpers.js"),
            "js/pages/admin-marketplace-versions-panel-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions-panel.js"),
            "js/pages/admin-marketplace-versions-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions.js"),
            "js/pages/admin-marketplace-versions.js",
        ),
    ]
}

fn page_admin_my_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-my-activity.js"),
            "js/pages/admin-my-activity.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-agent-edit.js"),
            "js/pages/admin-my-agent-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-agents.js"),
            "js/pages/admin-my-agents.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-hooks.js"),
            "js/pages/admin-my-hooks.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-marketplace.js"),
            "js/pages/admin-my-marketplace.js",
        ),
        AssetDefinition::js(pages.join("admin-my-mcp.js"), "js/pages/admin-my-mcp.js"),
        AssetDefinition::js(
            pages.join("admin-my-plugin-edit.js"),
            "js/pages/admin-my-plugin-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-plugins.js"),
            "js/pages/admin-my-plugins.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-secrets.js"),
            "js/pages/admin-my-secrets.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-skill-edit.js"),
            "js/pages/admin-my-skill-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-skills.js"),
            "js/pages/admin-my-skills.js",
        ),
    ]
}

fn page_admin_org_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-org-access-panel.js"),
            "js/pages/admin-org-access-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-access.js"),
            "js/pages/admin-org-access.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-events-helpers.js"),
            "js/pages/admin-org-agents-events-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-events.js"),
            "js/pages/admin-org-agents-events.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-panel.js"),
            "js/pages/admin-org-agents-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents.js"),
            "js/pages/admin-org-agents.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-hooks.js"),
            "js/pages/admin-org-hooks.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-marketplaces-panel.js"),
            "js/pages/admin-org-marketplaces-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-marketplaces.js"),
            "js/pages/admin-org-marketplaces.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-mcp-panel.js"),
            "js/pages/admin-org-mcp-panel.js",
        ),
        AssetDefinition::js(pages.join("admin-org-mcp.js"), "js/pages/admin-org-mcp.js"),
        AssetDefinition::js(
            pages.join("admin-org-skills-events.js"),
            "js/pages/admin-org-skills-events.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-skills-panel.js"),
            "js/pages/admin-org-skills-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-skills.js"),
            "js/pages/admin-org-skills.js",
        ),
    ]
}

fn page_admin_plugin_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-plugin-edit.js"),
            "js/pages/admin-plugin-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard-review.js"),
            "js/pages/admin-plugin-wizard-review.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard-steps.js"),
            "js/pages/admin-plugin-wizard-steps.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard.js"),
            "js/pages/admin-plugin-wizard.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugins-list.js"),
            "js/pages/admin-plugins-list.js",
        ),
    ]
}
