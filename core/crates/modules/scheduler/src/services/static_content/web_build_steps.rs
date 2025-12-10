use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

use super::web_build::{BuildError, BuildMode, Result};

pub async fn generate_theme(web_dir: &PathBuf) -> Result<()> {
    println!("\x1b[36m  -> Generating theme CSS and TypeScript config...\x1b[0m");

    let script_path = web_dir.join("scripts/generate-theme.js");

    if !script_path.exists() {
        return Err(BuildError::ThemeGenerationFailed(format!(
            "Theme generation script not found at: {}",
            script_path.display()
        )));
    }

    let output = Command::new("node")
        .current_dir(web_dir)
        .arg("scripts/generate-theme.js")
        .output()
        .await
        .map_err(|e| {
            BuildError::ProcessError(format!("Failed to execute theme generation: {e}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BuildError::ThemeGenerationFailed(format!(
            "Theme generation script failed:\n{stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout.trim());
    }

    println!("\x1b[32m  [OK] Theme generation complete\x1b[0m");
    Ok(())
}

pub async fn compile_typescript(web_dir: &PathBuf) -> Result<()> {
    println!("\x1b[36m  -> Compiling TypeScript...\x1b[0m");

    let output = Command::new("npx")
        .current_dir(web_dir)
        .args(["tsc", "-b"])
        .output()
        .await
        .map_err(|e| BuildError::ProcessError(format!("Failed to execute tsc: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(BuildError::TypeScriptFailed(format!(
            "TypeScript compilation failed:\nstdout: {stdout}\nstderr: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout.trim());
    }

    println!("\x1b[32m  [OK] TypeScript compilation complete\x1b[0m");
    Ok(())
}

pub async fn build_vite(web_dir: &PathBuf, mode: &BuildMode) -> Result<()> {
    let mode_str = mode.as_str();

    if matches!(mode, BuildMode::Production | BuildMode::Docker) {
        remove_env_local_if_exists(web_dir)?;
    }

    let env_vars = load_vite_env_vars(web_dir, mode_str)?;
    println!("\x1b[36m  -> Building with Vite (mode: {mode_str})...\x1b[0m");

    let mut command = Command::new("npx");
    command
        .current_dir(web_dir)
        .args(["vite", "build", "--mode", mode_str]);

    for (key, value) in &env_vars {
        command.env(key, value);
    }

    let output = command
        .output()
        .await
        .map_err(|e| BuildError::ProcessError(format!("Failed to execute vite: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(BuildError::ViteFailed(format!(
            "Vite build failed:\nstdout: {stdout}\nstderr: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout.trim());
    }

    println!("\x1b[32m  [OK] Vite build complete\x1b[0m");
    Ok(())
}

fn remove_env_local_if_exists(web_dir: &PathBuf) -> Result<()> {
    let env_local = web_dir.join(".env.local");
    if env_local.exists() {
        std::fs::remove_file(&env_local)
            .map_err(|e| BuildError::ProcessError(format!("Failed to remove .env.local: {e}")))?;
        println!("\x1b[36m  -> Removed .env.local to prevent override\x1b[0m");
    }
    Ok(())
}

fn load_vite_env_vars(web_dir: &PathBuf, mode_str: &str) -> Result<HashMap<String, String>> {
    let env_file = web_dir.join(format!(".env.{mode_str}"));
    let mut env_vars = HashMap::new();

    if !env_file.exists() {
        return Ok(env_vars);
    }

    let content = std::fs::read_to_string(&env_file)
        .map_err(|e| BuildError::ProcessError(format!("Failed to read env file: {e}")))?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            if key.starts_with("VITE_") {
                env_vars.insert(key.to_string(), value.to_string());
            }
        }
    }

    println!(
        "\x1b[36m  -> Loaded {} VITE variables from {}\x1b[0m",
        env_vars.len(),
        env_file.display()
    );

    Ok(env_vars)
}

pub async fn organize_css(web_dir: &PathBuf) -> Result<()> {
    println!("\x1b[36m  -> Organizing CSS files...\x1b[0m");

    let dist_dir = web_dir.join("dist");
    let css_dir = dist_dir.join("css");

    fs::create_dir_all(&css_dir).await.map_err(|e| {
        BuildError::CssOrganizationFailed(format!("Failed to create css directory: {e}"))
    })?;

    let css_files = vec!["blog.css", "syntax-highlight.css"];

    for file_name in css_files {
        let source = dist_dir.join(file_name);
        let dest = css_dir.join(file_name);

        if source.exists() {
            fs::copy(&source, &dest).await.map_err(|e| {
                BuildError::CssOrganizationFailed(format!(
                    "Failed to copy {file_name} to css/: {e}"
                ))
            })?;
            println!("    Copied {file_name} -> css/{file_name}");
        } else {
            println!("\x1b[33m    Warning: {file_name} not found, skipping\x1b[0m");
        }
    }

    println!("\x1b[32m  [OK] CSS organization complete\x1b[0m");
    Ok(())
}
