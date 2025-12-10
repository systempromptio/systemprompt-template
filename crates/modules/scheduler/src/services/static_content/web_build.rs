use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

use super::web_build_steps::{build_vite, compile_typescript, generate_theme, organize_css};
use super::web_build_validation::validate_build;

pub type Result<T> = std::result::Result<T, BuildError>;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Theme generation failed: {0}")]
    ThemeGenerationFailed(String),

    #[error("TypeScript compilation failed: {0}")]
    TypeScriptFailed(String),

    #[error("Vite build failed: {0}")]
    ViteFailed(String),

    #[error("CSS organization failed: {0}")]
    CssOrganizationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process execution error: {0}")]
    ProcessError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Development,
    Production,
    Docker,
}

impl BuildMode {
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Some(Self::Development),
            "production" | "prod" => Some(Self::Production),
            "docker" => Some(Self::Docker),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Docker => "docker",
        }
    }
}

#[derive(Debug)]
pub struct BuildOrchestrator {
    web_dir: PathBuf,
    mode: BuildMode,
}

impl BuildOrchestrator {
    #[must_use]
    pub const fn new(web_dir: PathBuf, mode: BuildMode) -> Self {
        Self { web_dir, mode }
    }

    pub async fn build(&self) -> Result<()> {
        let start_time = Instant::now();

        println!(
            "\n\x1b[1;36mBuilding web assets (mode: {})...\x1b[0m",
            self.mode.as_str()
        );

        let pb = ProgressBar::new(5);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
                .expect("Invalid progress bar template")
                .progress_chars("=>-"),
        );

        pb.set_message("Theme Generation");
        generate_theme(&self.web_dir).await?;
        pb.inc(1);

        pb.set_message("TypeScript Compilation");
        compile_typescript(&self.web_dir).await?;
        pb.inc(1);

        pb.set_message("Vite Build");
        build_vite(&self.web_dir, &self.mode).await?;
        pb.inc(1);

        pb.set_message("CSS Organization");
        organize_css(&self.web_dir).await?;
        pb.inc(1);

        pb.set_message("Validation");
        validate_build(&self.web_dir).await?;
        pb.inc(1);

        pb.finish_with_message("Build complete");

        let elapsed = start_time.elapsed();
        println!(
            "\n\x1b[1;32m[OK] Build successful in {:.2}s\x1b[0m",
            elapsed.as_secs_f64()
        );

        Ok(())
    }

    pub async fn build_theme_only(&self) -> Result<()> {
        println!("\n\x1b[1;36mGenerating theme...\x1b[0m");
        generate_theme(&self.web_dir).await?;
        println!("\x1b[1;32m[OK] Theme generation complete\x1b[0m");
        Ok(())
    }

    pub async fn validate_only(&self) -> Result<()> {
        println!("\n\x1b[1;36mValidating build...\x1b[0m");
        validate_build(&self.web_dir).await?;
        println!("\x1b[1;32m[OK] Validation complete\x1b[0m");
        Ok(())
    }
}
