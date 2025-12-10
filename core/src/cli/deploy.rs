use anyhow::{anyhow, Result};
use clap::Subcommand;
use std::env;
use std::path::PathBuf;
use systemprompt_core_system::services::{
    ConfigManager, ConfigValidator, DeployEnvironment, DeploymentOrchestrator,
};

#[derive(Subcommand)]
pub enum DeployCommands {
    /// Generate configuration for an environment
    GenerateConfig {
        /// Environment (local, docker-dev, production)
        #[arg(short = 'e', long)]
        environment: String,

        /// Output file path (optional)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Validate environment configuration
    ValidateConfig {
        /// Environment (local, docker-dev, production)
        #[arg(short = 'e', long)]
        environment: String,
    },

    /// Deploy to production
    Deploy {
        /// GCP VM name
        #[arg(long, default_value = "systemprompt-blog-vm")]
        vm_name: String,

        /// GCP zone
        #[arg(long, default_value = "us-east1-b")]
        zone: String,

        /// GCP project ID
        #[arg(long, default_value = "vast-nectar-453310-d7")]
        project_id: String,

        /// Health check URL
        #[arg(long, default_value = "https://tyingshoelaces.com/api/v1/health")]
        health_url: String,

        /// Skip validation
        #[arg(long)]
        skip_validation: bool,

        /// Skip build
        #[arg(long)]
        skip_build: bool,
    },
}

pub async fn handle_deploy_command(cmd: DeployCommands) -> Result<()> {
    match cmd {
        DeployCommands::GenerateConfig {
            environment,
            output,
        } => generate_config_command(environment, output).await,

        DeployCommands::ValidateConfig { environment } => {
            validate_config_command(environment).await
        },

        DeployCommands::Deploy {
            vm_name,
            zone,
            project_id,
            health_url,
            skip_validation,
            skip_build,
        } => {
            deploy_command(
                vm_name,
                zone,
                project_id,
                health_url,
                skip_validation,
                skip_build,
            )
            .await
        },
    }
}

async fn generate_config_command(environment: String, output: Option<String>) -> Result<()> {
    let env = DeployEnvironment::from_str(&environment)?;

    let project_root = get_project_root()?;
    let config_manager = ConfigManager::new(project_root.clone());

    let config = config_manager.generate_config(env).await?;

    let output_path = if let Some(output) = output {
        PathBuf::from(output)
    } else {
        project_root.join(format!(".env.{}", env.as_str()))
    };

    config_manager.write_env_file(&config, &output_path)?;
    config_manager.write_web_env_file(&config)?;

    Ok(())
}

async fn validate_config_command(environment: String) -> Result<()> {
    let env = DeployEnvironment::from_str(&environment)?;

    let project_root = get_project_root()?;
    let config_manager = ConfigManager::new(project_root.clone());

    let config = config_manager.generate_config(env).await?;

    ConfigValidator::validate(&config)?;

    println!();
    println!("✅ Configuration is valid and ready for deployment");

    Ok(())
}

async fn deploy_command(
    vm_name: String,
    zone: String,
    project_id: String,
    health_url: String,
    skip_validation: bool,
    skip_build: bool,
) -> Result<()> {
    use colored::Colorize;

    println!();
    println!("{}", "🚀 SystemPrompt Deployment".cyan().bold());
    println!();

    let project_root = get_project_root()?;
    let orchestrator = DeploymentOrchestrator::new(project_root.clone());

    let env_file = project_root.join(".env.production.deploy");

    orchestrator
        .generate_and_validate_config(DeployEnvironment::Production, &env_file)
        .await?;

    if !skip_validation {
        orchestrator.validate_services_config().await?;
    }

    let image_version = if !skip_build {
        let version = orchestrator
            .build_docker_images(DeployEnvironment::Production)
            .await?;

        orchestrator
            .push_docker_images(&project_id, &version)
            .await?;

        version
    } else {
        // If build is skipped, use latest tag
        "latest".to_string()
    };

    orchestrator
        .deploy_to_gcp(&vm_name, &zone, &project_id, &env_file, &image_version)
        .await?;

    let ssh_client =
        systemprompt_core_system::services::GcpSshClient::new(vm_name, zone, project_id);

    orchestrator
        .perform_health_checks(&health_url, &ssh_client)
        .await?;

    orchestrator
        .run_post_deployment_cleanup(&ssh_client)
        .await?;

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════"
            .green()
            .bold()
    );
    println!("{}", "  ✓ Deployment Complete!".green().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════"
            .green()
            .bold()
    );
    println!();

    Ok(())
}

fn get_project_root() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;

    if current_dir.join("infrastructure").exists() {
        Ok(current_dir)
    } else if current_dir.parent().and_then(|p| p.parent()).is_some() {
        let potential_root = current_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        if potential_root.join("infrastructure").exists() {
            Ok(potential_root.to_path_buf())
        } else {
            Err(anyhow!("Could not find project root directory"))
        }
    } else {
        Err(anyhow!("Could not find project root directory"))
    }
}
