use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::Path;

#[derive(Debug)]
pub struct GcpSshClient {
    vm_name: String,
    zone: String,
    project_id: String,
}

impl GcpSshClient {
    pub const fn new(vm_name: String, zone: String, project_id: String) -> Self {
        Self {
            vm_name,
            zone,
            project_id,
        }
    }

    pub async fn execute_command(&self, command: &str) -> Result<String> {
        let gcloud_command = format!(
            "gcloud compute ssh {} --zone={} --project={} --command=\"{}\"",
            self.vm_name, self.zone, self.project_id, command
        );

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&gcloud_command)
            .output()
            .await
            .context("Failed to execute gcloud ssh command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("SSH command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        println!(
            "{}",
            format!("   Uploading {} to VM...", local_path.display()).blue()
        );

        let gcloud_command = format!(
            "gcloud compute scp {} {}:{} --zone={} --project={}",
            local_path.display(),
            self.vm_name,
            remote_path,
            self.zone,
            self.project_id
        );

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&gcloud_command)
            .output()
            .await
            .context("Failed to upload file via gcloud scp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("File upload failed: {}", stderr));
        }

        println!("{}", "   ✓ File uploaded successfully".green());

        Ok(())
    }

    pub async fn check_connectivity(&self) -> Result<()> {
        println!("{}", "   Checking VM connectivity...".blue());

        let test_command = "echo VM is reachable";
        self.execute_command(test_command).await?;

        println!("{}", "   ✓ VM is reachable".green());

        Ok(())
    }

    pub async fn get_container_status(&self) -> Result<String> {
        let command = "cd /opt/systemprompt-blog && sudo PROJECT_ID=vast-nectar-453310-d7 \
                       docker-compose ps 2>&1";
        self.execute_command(command).await
    }

    pub async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: u32,
    ) -> Result<String> {
        let command = format!(
            "sudo docker logs --tail={} {} 2>&1",
            tail_lines, container_name
        );
        self.execute_command(&command).await
    }

    pub async fn pull_and_restart_containers(&self, image_version: &str) -> Result<()> {
        println!(
            "{}",
            format!("   Deploying version: {image_version}").blue()
        );

        // Step 1: Stop existing containers first to ensure clean state
        // Docker Compose v1 is notoriously bad at detecting image changes
        println!("{}", "   Stopping existing containers...".blue());
        let stop_command = "cd /opt/systemprompt-blog && sudo PROJECT_ID=vast-nectar-453310-d7 \
                            docker-compose down --remove-orphans";
        let _ = self.execute_command(stop_command).await;
        println!("{}", "   ✓ Containers stopped".green());

        // Step 2: Pull fresh images directly (bypasses compose caching issues)
        println!("{}", "   Pulling fresh images from registry...".blue());
        let pull_api = format!(
            "sudo docker pull gcr.io/vast-nectar-453310-d7/systemprompt-blog-api:{}",
            image_version
        );
        let pull_web = format!(
            "sudo docker pull gcr.io/vast-nectar-453310-d7/systemprompt-blog-web:{}",
            image_version
        );

        self.execute_command(&pull_api).await?;
        self.execute_command(&pull_web).await?;

        // Also pull :latest to keep it in sync
        self.execute_command(
            "sudo docker pull gcr.io/vast-nectar-453310-d7/systemprompt-blog-api:latest",
        )
        .await?;
        self.execute_command(
            "sudo docker pull gcr.io/vast-nectar-453310-d7/systemprompt-blog-web:latest",
        )
        .await?;
        println!("{}", "   ✓ Images pulled".green());

        // Step 3: Start containers with explicit versioned tag
        // Using --force-recreate ensures containers use the freshly pulled images
        println!("{}", "   Starting containers with new images...".blue());
        let deploy_command = format!(
            "cd /opt/systemprompt-blog && sudo PROJECT_ID=vast-nectar-453310-d7 IMAGE_TAG={} \
             docker-compose up -d --force-recreate web api",
            image_version
        );

        self.execute_command(&deploy_command).await?;

        // Step 4: Wait for API container to be healthy
        // Note: Web container (nginx) may exit after setup or have no health check
        println!("{}", "   Waiting for API container to be healthy...".blue());
        let max_attempts = 60;
        let mut healthy = false;

        for attempt in 1..=max_attempts {
            let check_cmd = "sudo docker inspect --format='{{.State.Health.Status}}' \
                             systemprompt-api 2>/dev/null || echo 'missing'";
            let result = self.execute_command(check_cmd).await.unwrap_or_default();
            let api_status = result.trim();

            if api_status == "healthy" {
                healthy = true;
                break;
            }

            println!(
                "{}",
                format!(
                    "   Attempt {}/{}: api={}",
                    attempt, max_attempts, api_status
                )
                .dimmed()
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }

        if !healthy {
            println!(
                "{}",
                "   ❌ API container failed to become healthy".red().bold()
            );
            let logs = self
                .get_container_logs("systemprompt-api", 50)
                .await
                .unwrap_or_default();
            println!("{}", "   API logs:".yellow());
            println!("{}", logs);
            return Err(anyhow!(
                "API container failed health check after {} attempts (3 minutes)",
                max_attempts
            ));
        }
        println!("{}", "   ✓ API container healthy".green());

        // Step 5: Verify the deployed image matches what we pulled
        println!("{}", "   Verifying deployed images...".blue());
        let verify_command = format!(
            "sudo docker inspect --format='{{{{.Config.Image}}}}' systemprompt-api | grep -q '{}' \
             && echo 'API OK' || echo 'API MISMATCH'",
            image_version
        );
        let verify_result = self
            .execute_command(&verify_command)
            .await
            .unwrap_or_default();
        if verify_result.contains("MISMATCH") {
            println!(
                "{}",
                "   ⚠ Warning: Deployed image version mismatch detected".yellow()
            );
        } else {
            println!("{}", "   ✓ Image versions verified".green());
        }

        // Step 5: Clean up old images to free disk space
        println!("{}", "   Cleaning up old images...".blue());
        let cleanup_command = "sudo docker image prune -af --filter 'until=24h'";
        let _ = self.execute_command(cleanup_command).await;
        println!("{}", "   ✓ Deployment complete".green());

        Ok(())
    }
}
