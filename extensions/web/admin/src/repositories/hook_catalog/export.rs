use std::path::Path;

use super::super::super::types::{HookCatalogEntry, HookDetail};
use super::CATEGORY_SYSTEM;

pub fn read_hook_template(
    services_path: &Path,
    hook_id: &str,
    template_name: &str,
) -> Result<Option<String>, anyhow::Error> {
    let tmpl_path = services_path
        .join("hooks")
        .join(hook_id)
        .join(template_name);
    if tmpl_path.exists() {
        Ok(Some(std::fs::read_to_string(&tmpl_path)?))
    } else {
        Ok(None)
    }
}

#[must_use]
pub fn render_tracking_script(
    template: &str,
    plugin_id: &str,
    token: &str,
    platform_url: &str,
) -> String {
    template
        .replace("{{plugin_id}}", plugin_id)
        .replace("{{token}}", token)
        .replace("{{platform_url}}", platform_url)
}

pub fn catalog_to_detail(entry: &HookCatalogEntry) -> HookDetail {
    HookDetail {
        id: entry.id.clone(),
        plugin_id: entry.plugins.first().cloned().unwrap_or_else(String::new),
        name: entry.name.clone(),
        description: entry.description.clone(),
        event: entry.event.clone(),
        matcher: entry.matcher.clone(),
        command: entry.command.clone(),
        is_async: entry.is_async,
        system: entry.category == CATEGORY_SYSTEM,
        visible_to: entry.visible_to.clone(),
        enabled: entry.enabled,
    }
}
