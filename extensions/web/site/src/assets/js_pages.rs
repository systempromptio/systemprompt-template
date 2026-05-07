use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! page_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/pages/", $name))
    };
}

pub(super) fn page_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let pages = storage_js.join("pages");
    vec![
        page_js!(&pages, "admin-access.js"),
        page_js!(&pages, "admin-devices.js"),
        page_js!(&pages, "admin-access-control.js"),
        page_js!(&pages, "admin-access-panel.js"),
        page_js!(&pages, "admin-agent-edit.js"),
        page_js!(&pages, "admin-agents-helpers.js"),
        page_js!(&pages, "admin-contexts.js"),
        page_js!(&pages, "admin-entity-access.js"),
        page_js!(&pages, "admin-plugins-list.js"),
        page_js!(&pages, "admin-profile.js"),
        page_js!(&pages, "admin-settings.js"),
        page_js!(&pages, "admin-skill-edit.js"),
        page_js!(&pages, "admin-users-actions.js"),
        page_js!(&pages, "admin-users.js"),
    ]
}
