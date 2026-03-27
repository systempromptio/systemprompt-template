use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use std::path::{Path, PathBuf};
use systemprompt::models::Config;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::admin::templates::helpers;
use crate::config_loader;

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileAdminTemplatesJob;

fn output_path_for(page_id: &str) -> String {
    let path = match page_id {
        "dashboard" => "admin/index.html",
        "plugin-create" => "admin/plugins/create/index.html",
        "plugin-edit" => "admin/plugins/edit/index.html",
        "skill-edit" => "admin/skills/edit/index.html",
        "agent-edit" => "admin/agents/edit/index.html",
        "hook-edit" => "admin/hooks/edit/index.html",
        "mcp-edit" => "admin/mcp-servers/edit/index.html",
        "user-detail" => "admin/user/index.html",
        "presentation" => "presentation/index.html",
        _ => return format!("admin/{page_id}/index.html"),
    };
    path.to_string()
}

fn discover_templates(templates_dir: &std::path::Path) -> Result<Vec<(String, String, String)>> {
    let mut pages = Vec::new();
    let entries = std::fs::read_dir(templates_dir)
        .with_context(|| format!("Failed to read templates dir: {}", templates_dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("hbs") {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let page_id = file_name.trim_end_matches(".hbs").to_string();
        let output_path = output_path_for(&page_id);

        pages.push((file_name.to_string(), page_id, output_path));
    }

    pages.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(pages)
}

impl CompileAdminTemplatesJob {
    pub async fn execute_compile() -> Result<JobResult> {
        let start_time = std::time::Instant::now();

        tracing::info!("Compile admin templates job started");

        let admin_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("storage")
            .join("files")
            .join("admin");
        let templates_dir = admin_dir.join("templates");
        let partials_dir = admin_dir.join("partials");
        let compiled_dir = admin_dir.join("compiled");

        let admin_output = compiled_dir.join("admin");
        if admin_output.exists() {
            tokio::fs::remove_dir_all(&admin_output)
                .await
                .with_context(|| {
                    format!("Failed to clean compiled dir: {}", admin_output.display())
                })?;
        }
        tokio::fs::create_dir_all(&compiled_dir)
            .await
            .with_context(|| {
                format!("Failed to create compiled dir: {}", compiled_dir.display())
            })?;

        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);

        register_partials_recursive(&mut hbs, &partials_dir, &partials_dir)?;
        helpers::register_helpers(&mut hbs);

        let site_url = Config::get().map_or_else(
            |e| {
                tracing::warn!(error = %e, "Config not available, using empty site_url");
                String::new()
            },
            |c| c.api_external_url.trim_end_matches('/').to_string(),
        );

        let branding_value = match config_loader::load_branding_config() {
            Ok(Some(b)) => serde_json::to_value(&b).unwrap_or_default(),
            _ => serde_json::Value::Null,
        };

        let pages = discover_templates(&templates_dir)?;

        tracing::info!(count = pages.len(), "Discovered admin templates");

        let mut compiled = 0u64;
        let mut failed = 0u64;

        let compile_ctx = CompilePageCtx {
            hbs: &hbs,
            templates_dir: &templates_dir,
            compiled_dir: &compiled_dir,
            site_url: &site_url,
            branding_value: &branding_value,
        };
        for (template_file, page_id, output_path) in &pages {
            match compile_page(
                &compile_ctx,
                template_file,
                page_id,
                output_path,
            )
            .await
            {
                Ok(()) => compiled += 1,
                Err(e) => {
                    tracing::error!(
                        template = %template_file,
                        output = %output_path,
                        error = %e,
                        error_chain = ?e,
                        "Failed to compile admin template"
                    );
                    failed += 1;
                }
            }
        }

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            compiled,
            failed,
            duration_ms,
            "Compile admin templates job completed"
        );

        Ok(JobResult::success()
            .with_stats(compiled, failed)
            .with_duration(duration_ms))
    }
}

fn register_partials_recursive(
    hbs: &mut Handlebars<'static>,
    dir: &Path,
    base: &Path,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read partials dir: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            register_partials_recursive(hbs, &path, base)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read partial: {}", path.display()))?;

            let rel = path.strip_prefix(base).unwrap_or(&path);
            let name = rel.with_extension("").to_string_lossy().replace('\\', "/");

            hbs.register_partial(&name, &content)
                .with_context(|| format!("Failed to register partial: {name}"))?;
        }
    }
    Ok(())
}

struct CompilePageCtx<'a> {
    hbs: &'a Handlebars<'a>,
    templates_dir: &'a std::path::Path,
    compiled_dir: &'a std::path::Path,
    site_url: &'a str,
    branding_value: &'a serde_json::Value,
}

async fn compile_page(
    ctx: &CompilePageCtx<'_>,
    template_file: &str,
    page_id: &str,
    output_path: &str,
) -> Result<()> {
    let template_content = tokio::fs::read_to_string(ctx.templates_dir.join(template_file))
        .await
        .with_context(|| format!("Failed to read template: {template_file}"))?;

    let mut data = json!({ "page": page_id, "site_url": ctx.site_url });
    if !ctx.branding_value.is_null() {
        data.as_object_mut()
            .expect("json object")
            .insert("branding".to_string(), ctx.branding_value.clone());
    }

    let rendered = ctx.hbs
        .render_template(&template_content, &data)
        .with_context(|| format!("Failed to render template: {template_file}"))?;

    let dest = ctx.compiled_dir.join(output_path);
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    tokio::fs::write(&dest, rendered)
        .await
        .with_context(|| format!("Failed to write compiled output: {}", dest.display()))?;

    tracing::debug!(
        template = template_file,
        output = %dest.display(),
        "Compiled admin template"
    );

    Ok(())
}

#[async_trait::async_trait]
impl Job for CompileAdminTemplatesJob {
    fn name(&self) -> &'static str {
        "compile_admin_templates"
    }

    fn description(&self) -> &'static str {
        "Compiles Handlebars admin templates with shared sidebar partial"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &JobContext) -> Result<JobResult> {
        Self::execute_compile().await
    }
}

systemprompt::traits::submit_job!(&CompileAdminTemplatesJob);
