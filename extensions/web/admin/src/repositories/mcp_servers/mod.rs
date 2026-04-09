mod mutations;
mod queries;

pub use mutations::*;
pub use queries::*;

use systemprompt_web_shared::error::MarketplaceError;
use std::path::Path;

fn find_mcp_file(
    mcp_dir: &Path,
    server_id: &str,
) -> Result<Option<std::path::PathBuf>, MarketplaceError> {
    if !mcp_dir.exists() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(mcp_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if config
            .get("mcp_servers")
            .and_then(|m| m.get(server_id))
            .is_some()
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
