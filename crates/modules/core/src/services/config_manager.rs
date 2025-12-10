use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use systemprompt_core_logging::CliService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployEnvironment {
    Local,
    DockerDev,
    Production,
}

impl DeployEnvironment {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::DockerDev => "docker-dev",
            Self::Production => "production",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "local" => Ok(Self::Local),
            "docker" | "docker-dev" => Ok(Self::DockerDev),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(anyhow!("Invalid environment: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    #[serde(flatten)]
    pub vars: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub environment: DeployEnvironment,
    pub variables: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ConfigManager {
    project_root: PathBuf,
    environments_dir: PathBuf,
}

impl ConfigManager {
    pub fn new(project_root: PathBuf) -> Self {
        let environments_dir = project_root.join("infrastructure/environments");
        Self {
            project_root,
            environments_dir,
        }
    }

    pub async fn generate_config(
        &self,
        environment: DeployEnvironment,
    ) -> Result<EnvironmentConfig> {
        CliService::info(&format!(
            "📝 Building configuration for environment: {}",
            environment.as_str()
        ));

        let base_config_path = self.environments_dir.join("base.yml");
        let env_config_path = self
            .environments_dir
            .join(environment.as_str())
            .join("config.yml");

        if !base_config_path.exists() {
            return Err(anyhow!(
                "Base config not found: {}",
                base_config_path.display()
            ));
        }

        if !env_config_path.exists() {
            return Err(anyhow!(
                "Environment config not found: {}",
                env_config_path.display()
            ));
        }

        self.load_secrets().await?;

        CliService::success(&format!(
            "   Parsing base config: {}",
            base_config_path.display()
        ));
        let base_vars = self.yaml_to_flat_map(&base_config_path)?;

        CliService::success(&format!(
            "   Parsing environment config: {}",
            env_config_path.display()
        ));
        let env_vars = self.yaml_to_flat_map(&env_config_path)?;

        let merged = self.merge_configs(base_vars, env_vars);

        let resolved = self.resolve_variables(merged)?;

        CliService::success("   Configuration generated successfully");

        Ok(EnvironmentConfig {
            environment,
            variables: resolved,
        })
    }

    async fn load_secrets(&self) -> Result<()> {
        let secrets_file = self.project_root.join(".env.secrets");

        if secrets_file.exists() {
            CliService::info(&format!(
                "   Loading secrets from: {}",
                secrets_file.display()
            ));
            let content = fs::read_to_string(&secrets_file)?;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    std::env::set_var(key.trim(), value.trim().trim_matches('"'));
                }
            }

            CliService::success("   Secrets loaded");
        } else {
            CliService::warning("   No .env.secrets file found");
        }

        Ok(())
    }

    fn yaml_to_flat_map(&self, yaml_path: &Path) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(yaml_path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let mut flat_map = HashMap::new();
        self.flatten_yaml(&yaml, String::new(), &mut flat_map);

        Ok(flat_map)
    }

    fn flatten_yaml(
        &self,
        value: &serde_yaml::Value,
        prefix: String,
        result: &mut HashMap<String, String>,
    ) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    if let Some(key_str) = k.as_str() {
                        let new_prefix = if prefix.is_empty() {
                            key_str.to_uppercase()
                        } else {
                            format!("{}_{}", prefix, key_str.to_uppercase())
                        };
                        self.flatten_yaml(v, new_prefix, result);
                    }
                }
            },
            serde_yaml::Value::Sequence(_) => {},
            _ => {
                if let Some(str_val) = value.as_str() {
                    result.insert(prefix, str_val.to_string());
                } else if let Some(num_val) = value.as_i64() {
                    result.insert(prefix, num_val.to_string());
                } else if let Some(bool_val) = value.as_bool() {
                    result.insert(prefix, bool_val.to_string());
                } else if let Some(float_val) = value.as_f64() {
                    result.insert(prefix, float_val.to_string());
                }
            },
        }
    }

    fn merge_configs(
        &self,
        base: HashMap<String, String>,
        env: HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = base;
        for (k, v) in env {
            merged.insert(k, v);
        }
        merged
    }

    fn resolve_variables(
        &self,
        mut vars: HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let var_regex = Regex::new(r"\$\{([^}:]+)(?::-(.*?))?\}")?;
        let max_passes = 5;

        for current_pass in 0..max_passes {
            let mut changes_made = false;

            for (key, value) in vars.clone() {
                if var_regex.is_match(&value) {
                    let resolved = self.resolve_value(&value, &vars, &var_regex)?;

                    if resolved != value {
                        vars.insert(key, resolved);
                        changes_made = true;
                    }
                }
            }

            if !changes_made {
                break;
            }

            if current_pass == max_passes - 1 && changes_made {
                let unresolved: Vec<_> = vars
                    .iter()
                    .filter(|(_, v)| var_regex.is_match(v))
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect();

                if !unresolved.is_empty() {
                    return Err(anyhow!(
                        "Failed to resolve after {} passes:\n{}",
                        max_passes,
                        unresolved.join("\n")
                    ));
                }
            }
        }

        Ok(vars)
    }

    fn resolve_value(
        &self,
        value: &str,
        vars: &HashMap<String, String>,
        var_regex: &Regex,
    ) -> Result<String> {
        let mut result = value.to_string();

        for cap in var_regex.captures_iter(value) {
            let full_match = cap
                .get(0)
                .ok_or_else(|| anyhow!("Regex capture group 0 missing"))?
                .as_str();
            let var_name = cap
                .get(1)
                .ok_or_else(|| anyhow!("Regex capture group 1 missing"))?
                .as_str();
            let default_value = cap.get(2).map(|m| m.as_str());

            let replacement = if let Some(env_value) = std::env::var(var_name).ok() {
                env_value
            } else if let Some(config_value) = vars.get(var_name) {
                config_value.clone()
            } else if let Some(default) = default_value {
                default.to_string()
            } else {
                full_match.to_string()
            };

            result = result.replace(full_match, &replacement);
        }

        Ok(result)
    }

    pub fn write_env_file(&self, config: &EnvironmentConfig, output_path: &Path) -> Result<()> {
        let mut lines: Vec<String> = config
            .variables
            .iter()
            .map(|(k, v)| {
                if v.contains(char::is_whitespace) {
                    format!("{}=\"{}\"", k, v)
                } else {
                    format!("{k}={v}")
                }
            })
            .collect();

        lines.sort();

        fs::write(output_path, lines.join("\n"))?;

        CliService::success(&format!(
            "Configuration written to: {}",
            output_path.display()
        ));

        let var_count = lines.len();
        CliService::info(&format!("   {} environment variables generated", var_count));

        Ok(())
    }

    pub fn write_web_env_file(&self, config: &EnvironmentConfig) -> Result<()> {
        let web_env_path = self
            .project_root
            .join("core/web")
            .join(format!(".env.{}", config.environment.as_str()));

        let vite_vars: Vec<String> = config
            .variables
            .iter()
            .filter(|(k, _)| k.starts_with("VITE_"))
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        if !vite_vars.is_empty() {
            fs::write(&web_env_path, vite_vars.join("\n"))?;
            CliService::success(&format!(
                "Web configuration written to: {}",
                web_env_path.display()
            ));

            if config.environment == DeployEnvironment::Local {
                let web_dir = self.project_root.join("core/web");
                let env_link = web_dir.join(".env");
                let target = ".env.local";

                #[cfg(unix)]
                {
                    use std::os::unix::fs as unix_fs;
                    if env_link.exists() {
                        fs::remove_file(&env_link)?;
                    }
                    unix_fs::symlink(target, &env_link)?;
                    CliService::success(&format!(
                        "Created symlink: {} -> {}",
                        env_link.display(),
                        target
                    ));
                }
            }

            if config.environment == DeployEnvironment::DockerDev {
                let vite_docker_path = self.project_root.join("core/web/.env.docker");
                fs::write(&vite_docker_path, vite_vars.join("\n"))?;
                CliService::success(&format!(
                    "Web configuration also written to: {}",
                    vite_docker_path.display()
                ));
            }
        } else {
            CliService::warning("No VITE_* variables found in configuration");
        }

        Ok(())
    }
}
