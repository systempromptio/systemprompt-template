use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SystemToolsServer {
    /// Allowed root directories for file operations
    pub(super) roots: Arc<RwLock<Vec<PathBuf>>>,
}

impl SystemToolsServer {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            roots: Arc::new(RwLock::new(roots)),
        }
    }

    /// Check if a path is within the allowed roots
    pub async fn validate_path(&self, path: &std::path::Path) -> Result<PathBuf, String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path '{}': {}", path.display(), e))?;

        let roots = self.roots.read().await;

        // SECURITY: Require at least one root to be configured
        if roots.is_empty() {
            return Err("No file roots configured. Set FILE_ROOT environment variable.".to_string());
        }

        for root in roots.iter() {
            if let Ok(root_canonical) = root.canonicalize() {
                if canonical.starts_with(&root_canonical) {
                    return Ok(canonical);
                }
            }
        }

        Err(format!(
            "Access denied: '{}' is outside allowed roots: {:?}",
            path.display(),
            roots.iter().map(|r| r.display().to_string()).collect::<Vec<_>>()
        ))
    }

    /// Check if a path would be valid (for paths that don't exist yet)
    pub async fn validate_parent_path(&self, path: &std::path::Path) -> Result<PathBuf, String> {
        let roots = self.roots.read().await;

        // SECURITY: Require at least one root to be configured
        if roots.is_empty() {
            return Err("No file roots configured. Set FILE_ROOT environment variable.".to_string());
        }

        // For new files, check the parent directory
        if let Some(parent) = path.parent() {
            if parent.exists() {
                let parent_canonical = parent
                    .canonicalize()
                    .map_err(|e| format!("Failed to resolve parent path: {}", e))?;

                for root in roots.iter() {
                    if let Ok(root_canonical) = root.canonicalize() {
                        if parent_canonical.starts_with(&root_canonical) {
                            return Ok(path.to_path_buf());
                        }
                    }
                }

                return Err(format!(
                    "Access denied: '{}' is outside allowed roots: {:?}",
                    path.display(),
                    roots.iter().map(|r| r.display().to_string()).collect::<Vec<_>>()
                ));
            }
        }

        Err(format!(
            "Parent directory does not exist for '{}'",
            path.display()
        ))
    }

    /// Update roots from MCP initialize request
    pub async fn set_roots(&self, roots: Vec<PathBuf>) {
        let mut current_roots = self.roots.write().await;
        *current_roots = roots;
    }

    /// Get current roots
    pub async fn get_roots(&self) -> Vec<PathBuf> {
        self.roots.read().await.clone()
    }
}
