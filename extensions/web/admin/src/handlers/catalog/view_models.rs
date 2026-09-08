//! View-model assembly for the catalog pages.
//!
//! Turns a loaded [`Catalog`] plus per-entity assignment counts into the
//! `Serialize` structs the templates consume. Loading lives in [`super::data`].

use std::collections::HashMap;

use systemprompt::identifiers::SkillId;

use crate::types::{ConfiguredHook, ENTITY_PLUGIN, ENTITY_SKILL};

use super::data::Catalog;
use super::visibility::{VisibilityView, carriers_of_plugin, carriers_of_skill, visibility_for};
use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::types::access_control::AccessControlRule;

// Why: What the visibility badge needs that the catalog walk does not carry:
// the declared marketplace audiences and the rules written against entities.
pub(super) struct VisibilityInput<'a> {
    pub(super) manifests: &'a [MarketplaceConfigSummary],
    pub(super) rules: &'a [AccessControlRule],
}

impl VisibilityInput<'_> {
    fn plugin(&self, plugin_id: &str) -> VisibilityView {
        let carriers = carriers_of_plugin(self.manifests, plugin_id);
        visibility_for(self.rules, ENTITY_PLUGIN, plugin_id, &carriers)
    }

    fn skill(&self, skill_id: &SkillId, plugin_ids: &[String]) -> VisibilityView {
        let carriers = carriers_of_skill(self.manifests, plugin_ids);
        visibility_for(self.rules, ENTITY_SKILL, skill_id.as_str(), &carriers)
    }
}
use super::view::{
    HookRef, LinkedEntity, PluginDetailData, PluginListRow, SkillDetailData, SkillListRow,
    matrix_url, mcp_url, plugin_url, skill_url,
};

// Why: the two-crumb trail every catalog detail page carries — the listing it
// came from, then itself. Shared so a rename of a listing is one edit.
fn trail(
    listing: &'static str,
    href: &'static str,
    current: &str,
) -> Vec<crate::handlers::ssr::types::BreadcrumbView> {
    vec![
        crate::handlers::ssr::types::BreadcrumbView::link(listing, href),
        crate::handlers::ssr::types::BreadcrumbView::current(current.to_owned()),
    ]
}

pub(super) fn plugin_rows(
    catalog: Catalog,
    counts: &HashMap<String, i64>,
    visibility: &VisibilityInput<'_>,
) -> Vec<PluginListRow> {
    catalog
        .plugins
        .into_iter()
        .map(|p| PluginListRow {
            visibility: visibility.plugin(&p.id),
            detail_url: plugin_url(&p.id),
            matrix_url: matrix_url(ENTITY_PLUGIN, &p.id),
            skills_count: p.skills.len(),
            mcp_count: p.mcp_servers.len(),
            agents_count: p.agents.len(),
            assignment_count: counts.get(&p.id).copied().unwrap_or(0),
            id: p.id,
            name: p.name,
            description: p.description,
            category: p.category,
            version: p.version,
            enabled: p.enabled,
            source_path: p.source_path,
        })
        .collect()
}

// Why: a plugin names its skills by id; the catalog is what knows their
// display names, and a skill a plugin names but the catalog has lost falls
// back to its id rather than disappearing from the plugin's member list.
fn plugin_skills(catalog: &Catalog, plugin: &crate::types::PluginDetail) -> Vec<LinkedEntity> {
    let names: HashMap<String, String> = catalog
        .skills
        .iter()
        .map(|s| (s.id.as_str().to_owned(), s.name.clone()))
        .collect();
    plugin
        .skills
        .iter()
        .map(|s| {
            let id = s.as_str().to_owned();
            LinkedEntity {
                name: names.get(&id).cloned().unwrap_or_else(|| id.clone()),
                url: skill_url(&id),
                id,
            }
        })
        .collect()
}

pub(super) fn plugin_detail(
    catalog: &Catalog,
    plugin_id: &str,
    assignment_count: i64,
) -> Option<PluginDetailData> {
    let plugin = catalog.plugins.iter().find(|p| p.id == plugin_id)?;
    let skills = plugin_skills(catalog, plugin);
    let mcp_servers = plugin
        .mcp_servers
        .iter()
        .map(|m| {
            let id = m.as_str().to_owned();
            LinkedEntity {
                name: id.clone(),
                url: mcp_url(&id),
                id,
            }
        })
        .collect::<Vec<_>>();
    let agents = plugin
        .agents
        .iter()
        .map(|a| {
            let id = a.as_str().to_owned();
            LinkedEntity {
                name: catalog
                    .agent_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| id.clone()),
                url: String::new(),
                id,
            }
        })
        .collect::<Vec<_>>();
    let hooks: Vec<HookRef> = catalog
        .hooks_by_plugin
        .get(&plugin.id)
        .map(|hs| hs.iter().map(hook_ref).collect())
        .unwrap_or_default();

    Some(PluginDetailData {
        breadcrumbs: trail("Plugins", "/admin/plugins", &plugin.name),
        page: "plugin-detail",
        title: plugin.name.clone(),
        matrix_url: matrix_url(ENTITY_PLUGIN, &plugin.id),
        assignment_count,
        skills_count: skills.len(),
        mcp_count: mcp_servers.len(),
        agents_count: agents.len(),
        hooks_count: hooks.len(),
        skills,
        mcp_servers,
        agents,
        hooks,
        id: plugin.id.clone(),
        name: plugin.name.clone(),
        description: plugin.description.clone(),
        version: plugin.version.clone(),
        category: plugin.category.clone(),
        enabled: plugin.enabled,
        author_name: plugin.author_name.clone(),
        keywords: plugin.keywords.clone(),
        roles: plugin.roles.clone(),
        source_path: plugin.source_path.clone(),
    })
}

fn hook_ref(h: &ConfiguredHook) -> HookRef {
    HookRef {
        id: h.id.clone(),
        event: h.event.clone(),
        matcher: h.matcher.clone(),
        command: h.command.clone(),
        is_async: h.is_async,
    }
}

pub(super) fn skill_rows(
    catalog: &Catalog,
    counts: &HashMap<String, i64>,
    visibility: &VisibilityInput<'_>,
) -> Vec<SkillListRow> {
    catalog
        .skills
        .iter()
        .map(|s| {
            let id = s.id.as_str().to_owned();
            let plugin_ids: Vec<String> = catalog
                .plugins_by_skill
                .get(&id)
                .map(|links| links.iter().map(|l| l.id.clone()).collect())
                .unwrap_or_default();
            SkillListRow {
                visibility: visibility.skill(&s.id, &plugin_ids),
                detail_url: skill_url(&id),
                matrix_url: matrix_url(ENTITY_SKILL, &id),
                assignment_count: counts.get(&id).copied().unwrap_or(0),
                plugin_count: catalog.plugins_by_skill.get(&id).map_or(0, Vec::len),
                id,
                name: s.name.clone(),
                description: s.description.clone(),
                enabled: s.enabled,
                source_path: s.source_path.clone(),
            }
        })
        .collect()
}

pub(super) fn skill_detail(
    catalog: &Catalog,
    skill: &SkillId,
    assignment_count: i64,
) -> Option<SkillDetailData> {
    let id = skill.as_str();
    let entry = catalog.skills.iter().find(|s| s.id == *skill)?;
    let included_by = catalog
        .plugins_by_skill
        .get(id)
        .cloned()
        .unwrap_or_default();
    Some(SkillDetailData {
        breadcrumbs: trail("Skills", "/admin/skills", &entry.name),
        // Why: the catalog page defines the skill; the analytics tab says who
        // actually runs it. They are different pages and this is the hop.
        activity_url: format!("/admin/analytics?tab=skills&skill={id}"),
        page: "skill-detail",
        title: entry.name.clone(),
        matrix_url: matrix_url(ENTITY_SKILL, id),
        assignment_count,
        included_by_count: included_by.len(),
        included_by,
        id: id.to_owned(),
        name: entry.name.clone(),
        description: entry.description.clone(),
        enabled: entry.enabled,
        source_path: entry.source_path.clone(),
    })
}
