use crate::handlers::ssr::types::PluginEditData;

pub(in crate::handlers::ssr) fn build_plugin_edit_data(
    plugin_with_assoc: Option<&crate::types::UserPluginWithAssociations>,
) -> PluginEditData {
    plugin_with_assoc.map_or_else(
        || PluginEditData {
            id: None,
            plugin_id: String::new(),
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            enabled: true,
            category: String::new(),
            keywords: vec![],
            author_name: String::new(),
            base_plugin_id: None,
        },
        |p| PluginEditData {
            id: Some(p.plugin.id.clone()),
            plugin_id: p.plugin.plugin_id.clone(),
            name: p.plugin.name.clone(),
            description: p.plugin.description.clone(),
            version: p.plugin.version.clone(),
            enabled: p.plugin.enabled,
            category: p.plugin.category.clone(),
            keywords: p.plugin.keywords.clone(),
            author_name: p.plugin.author_name.clone(),
            base_plugin_id: p.plugin.base_plugin_id.clone(),
        },
    )
}
