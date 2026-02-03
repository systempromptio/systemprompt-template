//! Build script for compile-time config validation.
//!
//! Validates YAML configuration files at compile time and sets up
//! rerun-if-changed directives for incremental builds.

use std::fs;
use std::path::Path;

fn main() {
    validate_yaml_exists("services/scheduler/config.yaml");
    validate_yaml_exists("services/web/config.yaml");
    validate_yaml_exists("services/content/config.yaml");

    validate_agent_configs();
    validate_mcp_configs();
}

fn validate_yaml_exists(path: &str) {
    println!("cargo:rerun-if-changed={path}");

    if !Path::new(path).exists() {
        println!("cargo:warning=Config file not found: {path}");
        return;
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                panic!("Invalid YAML in {path}: {e}");
            }
        }
        Err(e) => {
            panic!("Failed to read {path}: {e}");
        }
    }
}

fn validate_agent_configs() {
    let agents_dir = Path::new("services/agents");
    println!("cargo:rerun-if-changed=services/agents");

    if !agents_dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(agents_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            let path_str = path.display().to_string();
            println!("cargo:rerun-if-changed={path_str}");

            match fs::read_to_string(&path) {
                Ok(content) => {
                    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        panic!("Invalid YAML in {path_str}: {e}");
                    }
                }
                Err(e) => {
                    panic!("Failed to read {path_str}: {e}");
                }
            }
        }
    }
}

fn validate_mcp_configs() {
    let mcp_dir = Path::new("services/mcp");
    println!("cargo:rerun-if-changed=services/mcp");

    if !mcp_dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(mcp_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            let path_str = path.display().to_string();
            println!("cargo:rerun-if-changed={path_str}");

            match fs::read_to_string(&path) {
                Ok(content) => {
                    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        panic!("Invalid YAML in {path_str}: {e}");
                    }
                }
                Err(e) => {
                    panic!("Failed to read {path_str}: {e}");
                }
            }
        }
    }
}
