use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde_json::json;

use systemprompt_web_admin::templates::helpers;
use systemprompt_web_shared::error::MarketplaceError;

pub(super) async fn prepare_compiled_dir(compiled_dir: &Path) -> Result<(), MarketplaceError> {
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

pub(super) fn build_handlebars_registry(
    partials_dir: &Path,
) -> Result<Handlebars<'static>, MarketplaceError> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    register_partials_recursive(&mut hbs, partials_dir, partials_dir)?;
    helpers::register_helpers(&mut hbs);
    Ok(hbs)
}

pub(super) fn resolve_branding_value() -> serde_json::Value {
    let config_dir = systemprompt::models::AppPaths::get().map_or_else(
        |e| {
            tracing::warn!(error = %e, "Failed to get app paths, using fallback config dir");
            PathBuf::from("./services/config")
        },
        |p| p.system().services().join("config"),
    );
    let theme_path = config_dir.join("theme.yaml");
    let Ok(content) = std::fs::read_to_string(&theme_path) else {
        return serde_json::Value::Null;
    };
    let Ok(theme): Result<serde_yaml::Value, _> = serde_yaml::from_str(&content) else {
        return serde_json::Value::Null;
    };
    let Some(branding) = theme.get("branding") else {
        return serde_json::Value::Null;
    };
    let Ok(config) =
        serde_yaml::from_value::<systemprompt_web_shared::BrandingConfig>(branding.clone())
    else {
        return serde_json::Value::Null;
    };
    serde_json::to_value(&config).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize branding config to JSON");
        serde_json::Value::Null
    })
}

pub(super) async fn compile_all_pages(
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

pub(super) struct CompilePageCtx<'a> {
    pub(super) hbs: &'a Handlebars<'a>,
    pub(super) templates_dir: &'a Path,
    pub(super) compiled_dir: &'a Path,
    pub(super) site_url: &'a str,
    pub(super) branding_value: &'a serde_json::Value,
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
