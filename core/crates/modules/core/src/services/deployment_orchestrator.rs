use crate::services::config_manager::{ConfigManager, DeployEnvironment};
use crate::services::config_validator::ConfigValidator;
use crate::services::gcp_ssh::GcpSshClient;
use crate::services::health_checker::HealthChecker;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct DeploymentOrchestrator {
    project_root: PathBuf,
    config_manager: ConfigManager,
}

impl DeploymentOrchestrator {
    pub fn new(project_root: PathBuf) -> Self {
        let config_manager = ConfigManager::new(project_root.clone());

        Self {
            project_root,
            config_manager,
        }
    }

    pub async fn generate_and_validate_config(
        &self,
        environment: DeployEnvironment,
        output_path: &Path,
    ) -> Result<()> {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!("{}", "  Configuration Generation".cyan().bold());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!();

        let config = self.config_manager.generate_config(environment).await?;

        self.config_manager.write_env_file(&config, output_path)?;
        self.config_manager.write_web_env_file(&config)?;

        println!();
        ConfigValidator::validate(&config)?;

        Ok(())
    }

    pub async fn validate_services_config(&self) -> Result<()> {
        println!();
        println!("{}", "   Validating services config includes...".blue());

        let config_dir = self.project_root.join("crates/services/config");
        let config_file = config_dir.join("config.yml");

        if !config_file.exists() {
            return Err(anyhow!(
                "Services config not found: {}",
                config_file.display()
            ));
        }

        let content = std::fs::read_to_string(&config_file)?;
        let mut missing_files = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if line.starts_with('-') {
                let include_path = line
                    .trim_start_matches('-')
                    .trim()
                    .trim_start_matches("..")
                    .trim_start_matches('/')
                    .trim_start_matches('\\');

                let full_path = config_dir
                    .parent()
                    .unwrap_or(&config_dir)
                    .join(include_path);

                if !full_path.exists() {
                    missing_files.push(format!(
                        "{} (referenced as: {})",
                        full_path.display(),
                        include_path
                    ));
                }
            }
        }

        if !missing_files.is_empty() {
            println!();
            println!(
                "{}",
                "❌ FATAL: Services config references missing files:"
                    .red()
                    .bold()
            );
            for file in &missing_files {
                println!("  - {}", file);
            }
            println!();
            println!("Fix: Either create the missing files or remove them from config.yml");
            return Err(anyhow!("Services config validation failed"));
        }

        println!("{}", "   ✓ All services config includes exist".green());

        Ok(())
    }

    pub async fn build_docker_images(&self, environment: DeployEnvironment) -> Result<String> {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!("{}", "  Building Docker Images".cyan().bold());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!();

        let build_script = self.project_root.join("infrastructure/scripts/build.sh");

        if !build_script.exists() {
            return Err(anyhow!(
                "Build script not found: {}",
                build_script.display()
            ));
        }

        let image_version = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        println!("{}", format!("   Image version: {image_version}").blue());
        println!();

        let build_command = format!(
            "IMAGE_VERSION={} ENVIRONMENT={} {} release --web --docker",
            image_version,
            environment.as_str(),
            build_script.display()
        );

        println!("{}", "   Running: ".blue());
        println!("{}", format!("   {build_command}").dimmed());
        println!();
        println!(
            "{}",
            "─────────────────────────────────────────────────────".dimmed()
        );

        // Stream output in real-time using inherit
        let status = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&build_command)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await
            .context("Failed to execute build script")?;

        println!(
            "{}",
            "─────────────────────────────────────────────────────".dimmed()
        );
        println!();

        if !status.success() {
            println!("{}", "❌ FATAL: Docker build failed".red().bold());
            println!();
            println!("The build command exited with status: {}", status);
            return Err(anyhow!("Docker build failed with status: {}", status));
        }

        println!("{}", "✓ Docker images built successfully".green().bold());

        Ok(image_version)
    }

    pub async fn push_docker_images(&self, project_id: &str, image_version: &str) -> Result<()> {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!("{}", "  Pushing Docker Images".cyan().bold());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!();

        let images = vec![
            format!("gcr.io/{}/systemprompt-blog-api:latest", project_id),
            format!(
                "gcr.io/{}/systemprompt-blog-api:{}",
                project_id, image_version
            ),
            format!("gcr.io/{}/systemprompt-blog-web:latest", project_id),
            format!(
                "gcr.io/{}/systemprompt-blog-web:{}",
                project_id, image_version
            ),
        ];

        for image in &images {
            println!("{}", format!("   Pushing {}...", image).blue());
            println!(
                "{}",
                "─────────────────────────────────────────────────────".dimmed()
            );

            let status = tokio::process::Command::new("docker")
                .arg("push")
                .arg(image)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .await
                .context("Failed to push Docker image")?;

            println!(
                "{}",
                "─────────────────────────────────────────────────────".dimmed()
            );

            if !status.success() {
                return Err(anyhow!(
                    "Failed to push image {}: exit status {}",
                    image,
                    status
                ));
            }

            println!("{}", format!("   ✓ {} pushed successfully", image).green());
            println!();
        }

        println!("{}", "✓ All images pushed successfully".green().bold());

        Ok(())
    }

    pub async fn deploy_to_gcp(
        &self,
        vm_name: &str,
        zone: &str,
        project_id: &str,
        env_file: &Path,
        image_version: &str,
    ) -> Result<()> {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!("{}", "  Deploying to GCP".cyan().bold());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!();

        let ssh_client = GcpSshClient::new(
            vm_name.to_string(),
            zone.to_string(),
            project_id.to_string(),
        );

        ssh_client.check_connectivity().await?;

        println!("{}", "   Deploying configuration...".blue());
        ssh_client
            .upload_file(env_file, "/tmp/.env.production.new")
            .await?;

        ssh_client
            .execute_command(
                "sudo mv /tmp/.env.production.new /opt/systemprompt-blog/.env.production && sudo \
                 chown root:root /opt/systemprompt-blog/.env.production && sudo chmod 600 \
                 /opt/systemprompt-blog/.env.production",
            )
            .await?;
        println!("{}", "   ✓ Configuration deployed".green());

        println!("{}", "   Deploying docker-compose template...".blue());
        let compose_file = self
            .project_root
            .join("infrastructure/environments/production/docker-compose.yml");
        ssh_client
            .upload_file(&compose_file, "/tmp/docker-compose.yml.new")
            .await?;

        ssh_client
            .execute_command(
                "sudo mv /tmp/docker-compose.yml.new /opt/systemprompt-blog/docker-compose.yml && \
                 sudo chown root:root /opt/systemprompt-blog/docker-compose.yml && sudo chmod 644 \
                 /opt/systemprompt-blog/docker-compose.yml",
            )
            .await?;
        println!("{}", "   ✓ Docker compose template deployed".green());

        let geoip_db = self.project_root.join("data/GeoLite2-City.mmdb");
        if geoip_db.exists() {
            println!("{}", "   Deploying GeoIP database...".blue());
            ssh_client
                .upload_file(&geoip_db, "/tmp/GeoLite2-City.mmdb")
                .await?;

            ssh_client
                .execute_command(
                    "sudo mkdir -p /opt/systemprompt-blog/database && sudo mv \
                     /tmp/GeoLite2-City.mmdb /opt/systemprompt-blog/database/GeoLite2-City.mmdb \
                     && sudo chown root:root /opt/systemprompt-blog/database/GeoLite2-City.mmdb \
                     && sudo chmod 644 /opt/systemprompt-blog/database/GeoLite2-City.mmdb",
                )
                .await?;
            println!("{}", "   ✓ GeoIP database deployed".green());
        } else {
            println!(
                "{}",
                format!(
                    "   ⚠ Warning: GeoIP database not found at {}",
                    geoip_db.display()
                )
                .yellow()
            );
        }

        // Ensure images directory exists with proper permissions for generated images
        println!("{}", "   Ensuring images directory exists...".blue());
        ssh_client
            .execute_command(
                "sudo mkdir -p /opt/systemprompt-blog/images/generated_images && sudo chown -R \
                 1000:1000 /opt/systemprompt-blog/images && sudo chmod -R 755 \
                 /opt/systemprompt-blog/images",
            )
            .await?;
        println!("{}", "   ✓ Images directory configured".green());

        ssh_client
            .pull_and_restart_containers(image_version)
            .await?;

        println!();
        println!("{}", "   Container status:".blue());
        let status = ssh_client.get_container_status().await?;
        println!("{}", status);

        Ok(())
    }

    pub async fn perform_health_checks(&self, url: &str, ssh_client: &GcpSshClient) -> Result<()> {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!("{}", "  Health Checks".cyan().bold());
        println!(
            "{}",
            "═══════════════════════════════════════════════════"
                .cyan()
                .bold()
        );
        println!();

        let health_checker = HealthChecker::new(url.to_string());

        match health_checker.check().await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!();
                println!("{}", format!("❌ {e}").red().bold());
                println!();
                println!("{}", "🔍 Collecting diagnostics from remote VM...".bold());
                println!();

                println!("{}", "Remote Container Status:".yellow());
                match ssh_client.get_container_status().await {
                    Ok(status) if !status.trim().is_empty() => println!("{}", status),
                    Ok(_) => println!("  (no containers found or empty response)"),
                    Err(ssh_err) => println!("  (SSH failed: {})", ssh_err),
                }
                println!();

                println!("{}", "Remote API Container Logs (last 50 lines):".yellow());
                match ssh_client.get_container_logs("systemprompt-api", 50).await {
                    Ok(logs) if !logs.trim().is_empty() => println!("{}", logs),
                    Ok(_) => println!("  (no logs available)"),
                    Err(ssh_err) => println!("  (SSH failed: {})", ssh_err),
                }
                println!();

                println!("{}", "Troubleshooting tips:".cyan());
                println!(
                    "  1. Check VM SSH access: gcloud compute ssh systemprompt-blog-vm \
                     --zone=us-east1-b"
                );
                println!("  2. View full logs: just deploy-logs");
                println!("  3. Check container status on VM: sudo docker ps -a");
                println!("  4. Check API logs on VM: sudo docker logs systemprompt-api");

                Err(e)
            },
        }
    }

    pub async fn run_post_deployment_cleanup(&self, ssh_client: &GcpSshClient) -> Result<()> {
        println!();
        println!("{}", "   Running post-deployment cleanup...".blue());

        ssh_client
            .execute_command(
                "cd /opt/systemprompt-blog && sudo docker-compose exec -T api \
                 /app/target/release/systemprompt scheduler cleanup-sessions --hours 1",
            )
            .await?;

        println!("{}", "   ✓ Cleanup complete".green());

        Ok(())
    }
}
