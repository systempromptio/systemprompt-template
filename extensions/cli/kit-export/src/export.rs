//! Writes the kit tree.
//!
//! Plugins are laid out by core's own [`build_plugin_bundle`] — the same
//! projection the bridge serves — so skills, rules, hooks and auxiliary files
//! land where Claude Code's plugin contract expects them. Two adjustments:
//! `.mcp.json` is dropped (it names this instance's servers; the sidecar
//! references them by id instead); each `SKILL.md` is re-rendered from
//! `services/skills/<id>/` with `config.yaml` merged into its frontmatter
//! ([`crate::skill_md`]); and `plugin.json` keeps the authored version, not
//! the content-suffixed one the bundler stamps for the bridge.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use systemprompt::marketplace::{CatalogContent, build_plugin_bundle};
use systemprompt::models::services::{
    ComponentSource, MarketplaceConfig, PluginComponentRef, PluginConfig, PluginHooksRef,
    ServicesConfig,
};

// Why: the catalog loader wants an API URL for agent cards and managed MCP
// entries; neither reaches the kit, so any syntactically valid URL will do.
const PLACEHOLDER_API_URL: &str = "https://kit.invalid";
const MCP_FILE: &str = ".mcp.json";
const PLUGIN_MANIFEST: &str = ".claude-plugin/plugin.json";
const SIDECAR: &str = ".claude-plugin/systemprompt.yaml";
const MANIFEST: &str = ".claude-plugin/marketplace.json";

#[derive(Debug, Default)]
pub struct ExportReport {
    pub marketplace: String,
    pub plugins: Vec<String>,
    pub skills: BTreeSet<String>,
    pub files: usize,
}

impl ExportReport {
    pub fn summary(&self) -> String {
        format!(
            "exported marketplace {} — {} plugin(s), {} skill(s), {} file(s)",
            self.marketplace,
            self.plugins.len(),
            self.skills.len(),
            self.files
        )
    }
}

#[derive(Debug, Serialize)]
struct ComponentRefOut {
    source: ComponentSource,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include: Vec<String>,
}

impl From<&PluginComponentRef> for ComponentRefOut {
    fn from(r: &PluginComponentRef) -> Self {
        Self {
            source: r.source,
            include: r.include.clone(),
        }
    }
}

// Why: `schema: 1` and the sidecar's own key names, and nothing that
// `marketplace.json` already carries — core's import refuses `id`, `name`,
// `description`, `version`, `author`, `keywords`, `license`, `plugins` and
// `skills` in a sidecar. No `access`: who reaches a kit is declared in this
// repository's rules.yaml, never in the kit. Agents and artifacts are carried
// by id (core's sidecar accepts both), because this instance's marketplace
// and plugin reference them and the round trip would otherwise report them
// lost.
#[derive(Debug, Serialize)]
struct MarketplaceSidecarOut {
    schema: u32,
    marketplace: MarketplaceSidecarBody,
}

#[derive(Debug, Serialize)]
struct MarketplaceSidecarBody {
    title: String,
    visibility: systemprompt::models::services::MarketplaceVisibility,
    enabled: bool,
    mcp_servers: ComponentRefOut,
    agents: ComponentRefOut,
    artifacts: ComponentRefOut,
}

#[derive(Debug, Serialize)]
struct PluginSidecarOut {
    schema: u32,
    plugin: PluginSidecarBody,
}

#[derive(Debug, Serialize)]
struct PluginSidecarBody {
    title: String,
    category: String,
    enabled: bool,
    mcp_servers: ComponentRefOut,
    agents: ComponentRefOut,
    artifacts: ComponentRefOut,
    content_sources: ComponentRefOut,
    hooks: PluginHooksRef,
}

pub fn export_kit(
    services: &ServicesConfig,
    services_root: &Path,
    wanted: &str,
    out: &Path,
) -> Result<ExportReport> {
    let marketplace = services
        .marketplaces
        .iter()
        .find(|(id, _)| id.as_str() == wanted)
        .map(|(_, m)| m)
        .with_context(|| format!("no marketplace '{wanted}' in {}", services_root.display()))?;
    if marketplace.plugins.source != ComponentSource::Explicit {
        bail!("marketplace {wanted}: plugins.source must be explicit to export");
    }
    let catalog = CatalogContent::load(services, services_root, PLACEHOLDER_API_URL)
        .map_err(|e| anyhow::anyhow!("loading the catalog: {e}"))?;
    let content = catalog.as_content();

    let mut report = ExportReport {
        marketplace: wanted.to_owned(),
        ..ExportReport::default()
    };
    let mut entries = Vec::new();
    for plugin_id in &marketplace.plugins.include {
        let config = services
            .plugins
            .values()
            .find(|p| p.id.as_str() == plugin_id)
            .with_context(|| format!("plugin '{plugin_id}' is not defined"))?;
        let bundle = build_plugin_bundle(config, &content)
            .map_err(|e| anyhow::anyhow!("building plugin {plugin_id}: {e}"))?;
        let plugin_dir = out.join("plugins").join(plugin_id);
        for (rel, file) in &bundle {
            if rel == MCP_FILE {
                continue;
            }
            if let Some((skill_id, kebab)) = skill_id_of(rel) {
                let disk = services_root.join("skills").join(&skill_id);
                write(
                    &plugin_dir.join(rel),
                    crate::skill_md::render(&disk, &kebab)?.as_bytes(),
                )?;
                report.skills.insert(skill_id);
            } else if rel == PLUGIN_MANIFEST {
                write(
                    &plugin_dir.join(rel),
                    &authored_version(&file.bytes, &config.version)?,
                )?;
            } else {
                write(&plugin_dir.join(rel), &file.bytes)?;
            }
            report.files += 1;
        }
        write(
            &plugin_dir.join(SIDECAR),
            serde_yaml::to_string(&PluginSidecarOut {
                schema: 1,
                plugin: PluginSidecarBody {
                    title: config.name.clone(),
                    category: config.category.clone(),
                    enabled: config.enabled,
                    mcp_servers: (&config.mcp_servers).into(),
                    agents: (&config.agents).into(),
                    artifacts: (&config.artifacts).into(),
                    content_sources: (&config.content_sources).into(),
                    hooks: config.hooks.clone(),
                },
            })?
            .as_bytes(),
        )?;
        report.files += 1;
        entries.push(manifest_entry(config));
        report.plugins.push(plugin_id.clone());
    }

    write_marketplace_files(out, wanted, marketplace, &entries)?;
    report.files += 2;
    Ok(report)
}

// JSON: the marketplace.json plugin entries, Anthropic's outgoing shape.
fn write_marketplace_files(
    out: &Path,
    id: &str,
    marketplace: &MarketplaceConfig,
    entries: &[serde_json::Value],
) -> Result<()> {
    write(
        &out.join(MANIFEST),
        serde_json::to_string_pretty(&manifest(id, marketplace, entries))?.as_bytes(),
    )?;
    write(
        &out.join(SIDECAR),
        serde_yaml::to_string(&MarketplaceSidecarOut {
            schema: 1,
            marketplace: MarketplaceSidecarBody {
                title: marketplace.name.clone(),
                visibility: marketplace.visibility,
                enabled: marketplace.enabled,
                mcp_servers: (&marketplace.mcp_servers).into(),
                agents: (&marketplace.agents).into(),
                artifacts: (&marketplace.artifacts).into(),
            },
        })?
        .as_bytes(),
    )
}

fn skill_id_of(rel: &str) -> Option<(String, String)> {
    let rest = rel.strip_prefix("skills/")?;
    let (kebab, file) = rest.split_once('/')?;
    (file == "SKILL.md").then(|| (kebab.replace('-', "_"), kebab.to_owned()))
}

// JSON: `plugin.json` is Anthropic's format; only its `version` is touched.
fn authored_version(manifest: &[u8], version: &str) -> Result<Vec<u8>> {
    let mut doc: serde_json::Value = serde_json::from_slice(manifest)?;
    if let Some(obj) = doc.as_object_mut() {
        obj.insert(
            "version".to_owned(),
            serde_json::Value::String(version.to_owned()),
        );
    }
    Ok(serde_json::to_vec_pretty(&doc)?)
}

// JSON: `.claude-plugin/marketplace.json` is Anthropic's format — an outgoing
// fixed shape with no Rust type of its own on this side.
fn manifest_entry(config: &PluginConfig) -> serde_json::Value {
    serde_json::json!({
        "name": config.id.as_str(),
        "source": format!("./plugins/{}", config.id.as_str()),
        "description": config.description,
        "version": config.version,
        "category": config.category,
        "keywords": config.keywords,
        "author": { "name": config.author.name, "email": config.author.email },
        "license": config.license,
    })
}

fn manifest(
    id: &str,
    marketplace: &MarketplaceConfig,
    plugins: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "name": id,
        "owner": { "name": marketplace.author.name, "email": marketplace.author.email },
        "metadata": {
            "description": marketplace.description,
            "version": marketplace.version,
            "pluginRoot": "./plugins",
        },
        "plugins": plugins,
    })
}

fn write(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes).with_context(|| path.display().to_string())
}
