use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde_json::json;
use systemprompt::models::Config;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::admin::templates::helpers;
use crate::config_loader;
use crate::error::MarketplaceError;

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

fn discover_templates(
    templates_dir: &Path,
) -> Result<Vec<(String, String, String)>, MarketplaceError> {
    let mut pages = Vec::new();
    let entries = std::fs::read_dir(templates_dir).map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to read templates dir: {}: {e}",
            templates_dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            MarketplaceError::Internal(format!("Failed to read directory entry: {e}"))
        })?;
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
    pub async fn execute_compile() -> anyhow::Result<JobResult> {
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

        prepare_compiled_dir(&compiled_dir).await?;

        let hbs = build_handlebars_registry(&partials_dir)?;
        let site_url = resolve_site_url();
        let branding_value = resolve_branding_value();

        let pages = discover_templates(&templates_dir)?;
        tracing::info!(count = pages.len(), "Discovered admin templates");

        let compile_ctx = CompilePageCtx {
            hbs: &hbs,
            templates_dir: &templates_dir,
            compiled_dir: &compiled_dir,
            site_url: &site_url,
            branding_value: &branding_value,
        };

        let (compiled, failed) = compile_all_pages(&compile_ctx, &pages).await;
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

async fn prepare_compiled_dir(compiled_dir: &Path) -> Result<(), MarketplaceError> {
    let admin_output = compiled_dir.join("admin");
    if admin_output.exists() {
        tokio::fs::remove_dir_all(&admin_output)
            .await
            .map_err(|e| {
                MarketplaceError::Internal(format!(
                    "Failed to clean compiled dir: {}: {e}",
                    admin_output.display()
                ))
            })?;
    }
    tokio::fs::create_dir_all(compiled_dir).await.map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to create compiled dir: {}: {e}",
            compiled_dir.display()
        ))
    })?;
    Ok(())
}

fn build_handlebars_registry(partials_dir: &Path) -> Result<Handlebars<'static>, MarketplaceError> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    register_partials_recursive(&mut hbs, partials_dir, partials_dir)?;
    helpers::register_helpers(&mut hbs);
    Ok(hbs)
}

fn resolve_site_url() -> String {
    Config::get().map_or_else(
        |e| {
            tracing::warn!(error = %e, "Config not available, using empty site_url");
            String::new()
        },
        |c| c.api_external_url.trim_end_matches('/').to_string(),
    )
}

fn resolve_branding_value() -> serde_json::Value {
    match config_loader::load_branding_config() {
        Ok(Some(b)) => serde_json::to_value(&b).unwrap_or_default(),
        _ => serde_json::Value::Null,
    }
}

async fn compile_all_pages(
    compile_ctx: &CompilePageCtx<'_>,
    pages: &[(String, String, String)],
) -> (u64, u64) {
    let mut compiled = 0u64;
    let mut failed = 0u64;

    for (template_file, page_id, output_path) in pages {
        match compile_page(compile_ctx, template_file, page_id, output_path).await {
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

    (compiled, failed)
}

fn register_partials_recursive(
    hbs: &mut Handlebars<'static>,
    dir: &Path,
    base: &Path,
) -> Result<(), MarketplaceError> {
    if !dir.exists() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir).map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to read partials dir: {}: {e}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            register_partials_recursive(hbs, &path, base)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
            let content = std::fs::read_to_string(&path).map_err(|e| {
                MarketplaceError::Internal(format!(
                    "Failed to read partial: {}: {e}",
                    path.display()
                ))
            })?;

            let rel = path.strip_prefix(base).unwrap_or(&path);
            let name = rel.with_extension("").to_string_lossy().replace('\\', "/");

            hbs.register_partial(&name, &content).map_err(|e| {
                MarketplaceError::Internal(format!("Failed to register partial: {name}: {e}"))
            })?;
        }
    }
    Ok(())
}

struct CompilePageCtx<'a> {
    hbs: &'a Handlebars<'a>,
    templates_dir: &'a Path,
    compiled_dir: &'a Path,
    site_url: &'a str,
    branding_value: &'a serde_json::Value,
}

async fn compile_page(
    ctx: &CompilePageCtx<'_>,
    template_file: &str,
    page_id: &str,
    output_path: &str,
) -> Result<(), MarketplaceError> {
    let template_content = tokio::fs::read_to_string(ctx.templates_dir.join(template_file))
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!("Failed to read template: {template_file}: {e}"))
        })?;

    let mut data = json!({ "page": page_id, "site_url": ctx.site_url });
    if !ctx.branding_value.is_null() {
        if let Some(obj) = data.as_object_mut() {
            obj.insert("branding".to_string(), ctx.branding_value.clone());
        }
    }

    let rendered = ctx
        .hbs
        .render_template(&template_content, &data)
        .map_err(|e| {
            MarketplaceError::Internal(format!("Failed to render template: {template_file}: {e}"))
        })?;

    let dest = ctx.compiled_dir.join(output_path);
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to create directory: {}: {e}",
                parent.display()
            ))
        })?;
    }

    tokio::fs::write(&dest, rendered).await.map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to write compiled output: {}: {e}",
            dest.display()
        ))
    })?;

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

    async fn execute(&self, _ctx: &JobContext) -> anyhow::Result<JobResult> {
        Self::execute_compile().await
    }
}

systemprompt::traits::submit_job!(&CompileAdminTemplatesJob);
