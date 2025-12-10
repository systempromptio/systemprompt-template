use crate::models::modules::{Module, ModuleSchema, ModuleSeed};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use systemprompt_models::Config;

#[derive(Debug, Copy, Clone)]
pub struct BinaryPaths;

impl BinaryPaths {
    pub fn resolve_binary(binary_name: &str) -> Result<PathBuf> {
        let config = Config::global();

        if let Some(binary_dir) = &config.binary_dir {
            let binary_path = PathBuf::from(binary_dir).join(binary_name);
            if binary_path.exists() {
                return Ok(binary_path);
            }
            bail!(
                "Binary '{}' not found at configured path: {}\n\nRun: cargo build --bin {}",
                binary_name,
                binary_path.display(),
                binary_name
            );
        }

        let cargo_target_dir = &config.cargo_target_dir;

        let release_path = PathBuf::from(cargo_target_dir)
            .join("release")
            .join(binary_name);
        let debug_path = PathBuf::from(cargo_target_dir)
            .join("debug")
            .join(binary_name);

        let release_exists = release_path.exists();
        let debug_exists = debug_path.exists();

        match (release_exists, debug_exists) {
            (true, true) => {
                // Prefer debug binaries during development to ensure latest code is used
                Ok(debug_path)
            },
            (true, false) => Ok(release_path),
            (false, true) => Ok(debug_path),
            (false, false) => {
                bail!(
                    "Binary '{}' not found at either:\n  - {} (release)\n  - {} (debug)\n\nRun: \
                     cargo build --bin {}",
                    binary_name,
                    release_path.display(),
                    debug_path.display(),
                    binary_name
                )
            },
        }
    }

    pub fn binary_exists(binary_name: &str) -> bool {
        Self::resolve_binary(binary_name).is_ok()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ModulePaths;

impl ModulePaths {
    pub fn schema_path(module: &Module, schema: &ModuleSchema) -> Result<PathBuf> {
        let config = Config::global();
        let schema_path = Path::new(&config.system_path)
            .join("core")
            .join("crates")
            .join("modules")
            .join(&module.name)
            .join(&schema.file);

        if schema_path.exists() {
            Ok(schema_path)
        } else {
            bail!(
                "Schema file not found for module '{}': {}",
                module.name,
                schema_path.display()
            )
        }
    }

    pub fn seed_path(module: &Module, seed: &ModuleSeed) -> Result<PathBuf> {
        let config = Config::global();
        let seed_path = Path::new(&config.system_path)
            .join("core")
            .join("crates")
            .join("modules")
            .join(&module.name)
            .join(&seed.file);

        if seed_path.exists() {
            Ok(seed_path)
        } else {
            bail!(
                "Seed file not found for module '{}': {}",
                module.name,
                seed_path.display()
            )
        }
    }
}
