use anyhow::{Context, Result};
use rmcp::ServiceExt;
use std::env;
use std::path::PathBuf;
use system_tools::SystemToolsServer;

const DEFAULT_SERVICE_ID: &str = "system-tools";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let service_id = env::var("MCP_SERVICE_ID").unwrap_or_else(|_| {
        eprintln!("[INFO] MCP_SERVICE_ID not set, using default: {DEFAULT_SERVICE_ID}");
        DEFAULT_SERVICE_ID.to_string()
    });

    // Get file roots from environment (comma-separated) or use current directory
    // SECURITY: All file operations are restricted to these directories
    let roots: Vec<PathBuf> = env::var("FILE_ROOT")
        .map(|v| v.split(',').map(PathBuf::from).collect())
        .unwrap_or_else(|_| {
            let cwd = env::current_dir().unwrap();
            eprintln!("[WARN] FILE_ROOT not set, defaulting to current directory");
            vec![cwd]
        });

    eprintln!("[INFO] Allowed file roots:");
    for root in &roots {
        eprintln!("[INFO]   - {}", root.display());
    }
    eprintln!("[INFO] Starting System Tools MCP server '{service_id}' on stdio");

    let server = SystemToolsServer::new(roots);

    // Run with stdio transport (standard MCP transport)
    let service = server.serve(rmcp::transport::stdio()).await
        .context("Failed to start MCP server")?;

    // Wait for the service to complete
    service.waiting().await?;

    Ok(())
}
