mod helpers;

use std::path::PathBuf;

use systemprompt::models::Config;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_shared::error::MarketplaceError;

use helpers::{
    build_handlebars_registry, compile_all_pages, prepare_compiled_dir, resolve_branding_value,
    CompilePageCtx,
};

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
        _ => return format!("admin/{page_id}/index.html"),
    };
    path.to_string()
}

fn discover_templates(
    templates_dir: &std::path::Path,
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
    pub async fn execute_compile() -> Result<JobResult, MarketplaceError> {
        let start_time = std::time::Instant::now();

        tracing::info!("Compile admin templates job started");

        let admin_dir = std::env::current_dir()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to get current directory, using fallback");
                PathBuf::from(".")
            })
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

fn resolve_site_url() -> String {
    Config::get().map_or_else(
        |e| {
            tracing::warn!(error = %e, "Config not available, using empty site_url");
            String::new()
        },
        |c| c.api_external_url.trim_end_matches('/').to_string(),
    )
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
        Ok(Self::execute_compile().await?)
    }
}

systemprompt::traits::submit_job!(&CompileAdminTemplatesJob);
