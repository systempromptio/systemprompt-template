use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;
use systemprompt_core_scheduler::{BuildMode, BuildOrchestrator};

#[derive(Parser)]
#[command(name = "systemprompt-build")]
#[command(about = "SystemPrompt build system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build web assets
    Web {
        /// Build mode (development, production, docker)
        #[arg(long, short, default_value = "development")]
        mode: String,

        /// Web directory path (auto-detected: ./web or ./core/web)
        #[arg(long)]
        web_dir: Option<PathBuf>,
    },
    /// Generate theme CSS and TypeScript only
    Theme {
        /// Web directory path (auto-detected: ./web or ./core/web)
        #[arg(long)]
        web_dir: Option<PathBuf>,
    },
    /// Validate build output
    Validate {
        /// Web directory path (auto-detected: ./web or ./core/web)
        #[arg(long)]
        web_dir: Option<PathBuf>,
    },
}

fn detect_web_dir(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(dir) = explicit {
        return dir;
    }

    // Try ./web first (running from core/)
    let web = PathBuf::from("web");
    if web.join("scripts/generate-theme.js").exists() {
        return web;
    }

    // Try ./core/web (running from repo root)
    let core_web = PathBuf::from("core/web");
    if core_web.join("scripts/generate-theme.js").exists() {
        return core_web;
    }

    // Default fallback
    web
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Web { mode, web_dir } => {
            let web_dir = detect_web_dir(web_dir);
            let build_mode = match BuildMode::from_str(&mode) {
                Some(m) => m,
                None => {
                    eprintln!(
                        "\x1b[31mError: Invalid build mode '{}'. Valid options: development, \
                         production, docker\x1b[0m",
                        mode
                    );
                    process::exit(1);
                },
            };

            let orchestrator = BuildOrchestrator::new(web_dir, build_mode);
            orchestrator.build().await
        },
        Commands::Theme { web_dir } => {
            let web_dir = detect_web_dir(web_dir);
            let orchestrator = BuildOrchestrator::new(web_dir, BuildMode::Development);
            orchestrator.build_theme_only().await
        },
        Commands::Validate { web_dir } => {
            let web_dir = detect_web_dir(web_dir);
            let orchestrator = BuildOrchestrator::new(web_dir, BuildMode::Development);
            orchestrator.validate_only().await
        },
    };

    match result {
        Ok(()) => {
            process::exit(0);
        },
        Err(e) => {
            eprintln!("\n\x1b[1;31mBuild failed: {e}\x1b[0m");
            eprintln!("\n\x1b[33mTroubleshooting:\x1b[0m");
            eprintln!("  • Ensure Node.js and npm are installed");
            eprintln!("  • Run 'npm ci' in core/web to install dependencies");
            eprintln!("  • Check that all required files exist");
            eprintln!("  • Review error messages above for details");
            process::exit(1);
        },
    }
}
