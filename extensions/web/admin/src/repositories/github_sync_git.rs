use std::path::Path;

use anyhow::Result;

pub fn git_clone_shallow(url: &str, target: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["clone", "--depth", "1", url, "."])
        .current_dir(target)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git clone failed: {stderr}");
    }
    Ok(())
}

pub fn git_pull(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git pull failed: {stderr}");
    }
    Ok(())
}

pub fn git_head_hash(repo_path: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rev-parse HEAD failed: {stderr}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(super) fn git_add_all(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed: {stderr}");
    }
    Ok(())
}

pub(super) fn git_has_changes(repo_path: &Path) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git status failed: {stderr}");
    }
    Ok(!output.stdout.is_empty())
}

pub(super) fn git_commit(repo_path: &Path, message: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git commit failed: {stderr}");
    }
    Ok(())
}

pub(super) fn git_push(repo_path: &Path, remote_url: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote_url, "HEAD"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git push failed: {stderr}");
    }
    Ok(())
}

pub(super) fn build_authenticated_url(repo_url: &str) -> String {
    let token = std::env::var("GITHUB_MARKETPLACE_TOKEN").unwrap_or_else(|e| {
        tracing::debug!(error = %e, "GITHUB_MARKETPLACE_TOKEN env var not set");
        String::new()
    });
    if token.is_empty() {
        return repo_url.to_string();
    }

    repo_url.strip_prefix("https://").map_or_else(
        || repo_url.to_string(),
        |rest| format!("https://{token}@{rest}"),
    )
}
