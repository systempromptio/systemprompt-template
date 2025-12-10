use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    #[serde(skip_deserializing, default = "generate_uuid")]
    pub uuid: String,
    pub name: String,
    pub version: String,
    pub display_name: String,
    pub description: Option<String>,
    pub weight: Option<i32>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub schemas: Option<Vec<ModuleSchema>>,
    pub seeds: Option<Vec<ModuleSeed>>,
    pub permissions: Option<Vec<ModulePermission>>,
    #[serde(default)]
    pub audience: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub enabled: bool,
    #[serde(default)]
    pub api: Option<ApiConfig>,
}

fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub type ModuleDefinition = Module;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    #[serde(default)]
    pub path_prefix: Option<String>,
    #[serde(default)]
    pub openapi_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSchema {
    pub file: String,
    pub table: String,
    pub required_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSeed {
    pub file: String,
    pub table: String,
    pub check_column: String,
    pub check_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePermission {
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceCategory {
    Core,
    Agent,
    Mcp,
    Meta,
}

impl ServiceCategory {
    pub const fn base_path(&self) -> &'static str {
        match self {
            Self::Core => "/api/v1/core",
            Self::Agent => "/api/v1/agents",
            Self::Mcp => "/api/v1/mcp",
            Self::Meta => "/",
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Agent => "Agent",
            Self::Mcp => "MCP",
            Self::Meta => "Meta",
        }
    }

    pub fn mount_path(&self, module_name: &str) -> String {
        if module_name.is_empty() {
            self.base_path().to_string()
        } else {
            match self {
                Self::Meta => {
                    format!("/{module_name}")
                },
                _ => {
                    format!("{}/{}", self.base_path(), module_name)
                },
            }
        }
    }

    pub fn matches_path(&self, path: &str) -> bool {
        let base = self.base_path();
        if base == "/" {
            path == "/" || path.starts_with("/.well-known") || path.starts_with("/api/v1/meta")
        } else {
            path.starts_with(base)
        }
    }

    pub const fn all() -> &'static [Self] {
        &[Self::Core, Self::Agent, Self::Mcp, Self::Meta]
    }

    pub fn from_path(path: &str) -> Option<Self> {
        for category in &[Self::Core, Self::Agent, Self::Mcp] {
            if category.matches_path(path) {
                return Some(*category);
            }
        }
        if Self::Meta.matches_path(path) {
            Some(Self::Meta)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    Regular,
    Proxy,
}

#[derive(Clone, Debug)]
pub struct Modules {
    modules: Vec<Module>,
}

impl Modules {
    pub fn load(system_path: &str) -> Result<Self> {
        let modules = Self::scan_and_load(system_path)?;
        Ok(Self { modules })
    }

    pub const fn all(&self) -> &Vec<Module> {
        &self.modules
    }

    pub fn get(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.name == name)
    }

    fn scan_and_load(system_path: &str) -> Result<Vec<Module>> {
        // Try core/crates first (systemprompt-blog layout), then fall back to crates/
        // (systemprompt-os layout)
        let crates_dir = {
            let core_crates = Path::new(system_path).join("core").join("crates");
            if core_crates.exists() {
                core_crates
            } else {
                Path::new(system_path).join("crates")
            }
        };

        let mut modules = Vec::new();

        if !crates_dir.exists() {
            eprintln!("Crates directory not found: {}", crates_dir.display());
            return Ok(modules);
        }

        let module_categories = ["modules", "mcp", "agent"];

        for category in &module_categories {
            let category_dir = crates_dir.join(category);

            if !category_dir.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(category_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                if entry.file_name() == "module.yml" {
                    match Self::load_module_yaml(entry.path()) {
                        Ok(module) => {
                            modules.push(module);
                        },
                        Err(e) => {
                            eprintln!("Error parsing {}: {}", entry.path().display(), e);
                        },
                    }
                }
            }
        }

        modules.sort_by_key(|m| m.weight.unwrap_or(100));

        let modules = Self::resolve_dependencies(modules)?;

        Ok(modules)
    }

    fn resolve_dependencies(mut modules: Vec<Module>) -> Result<Vec<Module>> {
        use std::collections::HashSet;

        let mut ordered = Vec::new();
        let mut processed = HashSet::new();

        while !modules.is_empty() {
            let mut to_process = Vec::new();

            for module in &modules {
                let deps_satisfied = module
                    .dependencies
                    .iter()
                    .all(|dep| processed.contains(dep.as_str()));

                if deps_satisfied {
                    to_process.push(module.clone());
                }
            }

            if to_process.is_empty() && !modules.is_empty() {
                let remaining: Vec<_> = modules.iter().map(|m| m.name.clone()).collect();
                return Err(anyhow::anyhow!(
                    "Circular dependency detected in modules: {remaining:?}"
                ));
            }

            for module in to_process {
                ordered.push(module.clone());
                processed.insert(module.name.clone());
            }

            modules.retain(|module| !processed.contains(module.name.as_str()));
        }

        Ok(ordered)
    }

    fn load_module_yaml(path: &Path) -> Result<Module> {
        let content = std::fs::read_to_string(path)?;
        let module: Module = serde_yaml::from_str(&content)?;
        Ok(module)
    }

    pub fn list_names(&self) -> Vec<String> {
        self.modules.iter().map(|m| m.name.clone()).collect()
    }

    pub fn get_provided_audiences() -> Vec<String> {
        vec!["a2a".to_string(), "api".to_string(), "mcp".to_string()]
    }

    pub fn get_valid_audiences(&self, module_name: &str) -> Vec<String> {
        if let Some(module) = self.get(module_name) {
            module.audience.clone()
        } else {
            Self::get_provided_audiences()
        }
    }

    pub fn get_server_audiences(_server_name: &str, _port: u16) -> Vec<String> {
        Self::get_provided_audiences()
    }
}
