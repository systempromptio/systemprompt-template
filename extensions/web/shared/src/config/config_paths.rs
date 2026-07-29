//! Content-source validation and profile-aware path resolution for the
//! `services/content/` config model.

use std::path::{Path, PathBuf};

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::AppPaths;

use super::{ContentSourceRaw, ContentSourceValidated, ExtensionConfigErrors};

pub(super) fn validate_content_source(
    src: ContentSourceRaw,
    index: usize,
    base_path: &Path,
    errors: &mut ExtensionConfigErrors,
) -> Option<ContentSourceValidated> {
    let field_prefix = format!("content_sources[{index}]");

    if src.source_id.as_str().trim().is_empty() {
        errors.push(
            format!("{field_prefix}.source_id"),
            "source_id cannot be empty",
        );
        return None;
    }

    if src.category_id.as_str().trim().is_empty() {
        errors.push(
            format!("{field_prefix}.category_id"),
            "category_id cannot be empty",
        );
        return None;
    }

    let resolved_path = resolve_content_source_path(&src.path, base_path);

    if src.enabled {
        let source_id = &src.source_id;
        if !resolved_path.exists() {
            errors.push_with_path(
                format!("{field_prefix}.path"),
                format!("Content source '{source_id}' path does not exist"),
                &resolved_path,
            );
            return None;
        }

        if !resolved_path.is_dir() {
            errors.push_with_path(
                format!("{field_prefix}.path"),
                format!("Content source '{source_id}' path is not a directory"),
                &resolved_path,
            );
            return None;
        }
    }

    let canonical_path = resolved_path.canonicalize().unwrap_or(resolved_path);

    Some(ContentSourceValidated {
        source_id: src.source_id,
        category_id: src.category_id,
        path: canonical_path,
        allowed_content_types: src.allowed_content_types,
        enabled: src.enabled,
        override_existing: src.override_existing,
    })
}
pub(super) fn resolve_blog_config_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("BLOG_CONFIG") {
        return PathBuf::from(override_path);
    }
    ProfileBootstrap::get()
        .map_err(|e| e.to_string())
        .and_then(|profile| {
            AppPaths::from_profile(&profile.paths, profile.path_resolution())
                .map_err(|e| e.to_string())
        })
        .map_or_else(
            |_| PathBuf::from("./services/config/blog.yaml"),
            |paths| paths.system().services().join("config/blog.yaml"),
        )
}
pub(super) fn resolve_content_source_path(path: &str, base_path: &Path) -> PathBuf {
    if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else if path.starts_with("./") {
        let services_dir = ProfileBootstrap::get()
            .map_err(|e| e.to_string())
            .and_then(|profile| AppPaths::from_profile(&profile.paths, profile.path_resolution()).map_err(|e| e.to_string()))
            .map_or_else(
                |e| {
                    tracing::warn!(error = %e, "Failed to get app paths, using fallback services dir");
                    PathBuf::from("./services")
                },
                |p| p.system().services().to_path_buf(),
            );
        let clean_path = path.strip_prefix("./services/").unwrap_or(path);
        services_dir.join(clean_path)
    } else {
        base_path.join(path)
    }
}
