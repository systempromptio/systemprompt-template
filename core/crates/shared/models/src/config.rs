use anyhow::Result;
use std::env::VarError;
use std::sync::OnceLock;
use systemprompt_traits::ConfigProvider;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Environment {
    pub fn detect() -> Self {
        if let Ok(env) = std::env::var("SYSTEMPROMPT_ENV") {
            return Self::from_string(&env);
        }

        if let Ok(env) = std::env::var("RAILWAY_ENVIRONMENT") {
            if env == "production" {
                return Self::Production;
            }
        }

        if let Ok(env) = std::env::var("NODE_ENV") {
            return Self::from_string(&env);
        }

        if std::env::var("DOCKER_CONTAINER").is_ok() {
            return Self::Production;
        }

        if cfg!(debug_assertions) {
            return Self::Development;
        }

        Self::Production
    }

    fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Self::Development,
            "test" | "testing" => Self::Test,
            _ => Self::Production,
        }
    }

    pub const fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }

    pub const fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub const fn is_test(&self) -> bool {
        matches!(self, Self::Test)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::detect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbosityLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

impl VerbosityLevel {
    pub const fn from_environment(env: Environment) -> Self {
        match env {
            Environment::Development => Self::Verbose,
            Environment::Production => Self::Quiet,
            Environment::Test => Self::Normal,
        }
    }

    pub fn from_env_var() -> Option<Self> {
        if std::env::var("SYSTEMPROMPT_QUIET").ok().as_deref() == Some("1") {
            return Some(Self::Quiet);
        }

        if std::env::var("SYSTEMPROMPT_VERBOSE").ok().as_deref() == Some("1") {
            return Some(Self::Verbose);
        }

        if std::env::var("SYSTEMPROMPT_DEBUG").ok().as_deref() == Some("1") {
            return Some(Self::Debug);
        }

        if let Ok(level) = std::env::var("SYSTEMPROMPT_LOG_LEVEL") {
            return match level.to_lowercase().as_str() {
                "quiet" => Some(Self::Quiet),
                "normal" => Some(Self::Normal),
                "verbose" => Some(Self::Verbose),
                "debug" => Some(Self::Debug),
                _ => None,
            };
        }

        None
    }

    pub fn resolve() -> Self {
        if let Some(level) = Self::from_env_var() {
            return level;
        }

        let env = Environment::detect();
        Self::from_environment(env)
    }

    pub const fn is_quiet(&self) -> bool {
        matches!(self, Self::Quiet)
    }

    pub const fn is_verbose(&self) -> bool {
        matches!(self, Self::Verbose | Self::Debug)
    }

    pub const fn should_show_verbose(&self) -> bool {
        matches!(self, Self::Verbose | Self::Debug)
    }

    pub const fn should_log_to_db(&self) -> bool {
        !matches!(self, Self::Quiet)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub oauth_public_per_second: u64,
    pub oauth_auth_per_second: u64,
    pub contexts_per_second: u64,
    pub tasks_per_second: u64,
    pub artifacts_per_second: u64,
    pub agent_registry_per_second: u64,
    pub agents_per_second: u64,
    pub mcp_registry_per_second: u64,
    pub mcp_per_second: u64,
    pub stream_per_second: u64,
    pub content_per_second: u64,
    pub burst_multiplier: u64,
    pub disabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            oauth_public_per_second: 2,
            oauth_auth_per_second: 2,
            contexts_per_second: 50,
            tasks_per_second: 10,
            artifacts_per_second: 15,
            agent_registry_per_second: 20,
            agents_per_second: 3,
            mcp_registry_per_second: 20,
            mcp_per_second: 100,
            stream_per_second: 1,
            content_per_second: 20,
            burst_multiplier: 2,
            disabled: false,
        }
    }
}

impl RateLimitConfig {
    pub fn production() -> Self {
        Self::default()
    }

    pub const fn testing() -> Self {
        Self {
            oauth_public_per_second: 10000,
            oauth_auth_per_second: 10000,
            contexts_per_second: 10000,
            tasks_per_second: 10000,
            artifacts_per_second: 10000,
            agent_registry_per_second: 10000,
            agents_per_second: 10000,
            mcp_registry_per_second: 10000,
            mcp_per_second: 10000,
            stream_per_second: 10000,
            content_per_second: 10000,
            burst_multiplier: 100,
            disabled: false,
        }
    }

    pub const fn disabled() -> Self {
        let mut config = Self::testing();
        config.disabled = true;
        config
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("RATE_LIMIT_DISABLED") {
            config.disabled = val.to_lowercase() == "true";
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_OAUTH_PUBLIC_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.oauth_public_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_OAUTH_AUTH_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.oauth_auth_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_CONTEXTS_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.contexts_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_TASKS_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.tasks_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_ARTIFACTS_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.artifacts_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_AGENT_REGISTRY_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.agent_registry_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_AGENTS_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.agents_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_MCP_REGISTRY_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.mcp_registry_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_MCP_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.mcp_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_STREAM_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.stream_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_CONTENT_PER_SECOND") {
            if let Ok(n) = val.parse() {
                config.content_per_second = n;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_BURST_MULTIPLIER") {
            if let Ok(n) = val.parse() {
                config.burst_multiplier = n;
            }
        }

        config
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub sitename: String,
    pub database_type: String,
    pub database_url: String,
    pub github_link: String,
    pub github_token: Option<String>,
    pub system_path: String,
    pub host: String,
    pub port: u16,
    pub admin_password: String,
    pub api_server_url: String,
    pub api_external_url: String,
    pub jwt_issuer: String,
    pub jwt_secret: String,
    pub jwt_access_token_expiration: i64,
    pub jwt_refresh_token_expiration: i64,
    pub dangerously_bypass_oauth: bool,
    pub use_https: bool,
    pub cargo_target_dir: String,
    pub binary_dir: Option<String>,
    pub rate_limits: RateLimitConfig,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn init() -> Result<()> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let config = Self::from_env()?;
        CONFIG
            .set(config)
            .map_err(|_| anyhow::anyhow!("Config already initialized"))?;
        Ok(())
    }

    pub fn global() -> &'static Self {
        CONFIG
            .get()
            .expect("Config not initialized. Call Config::init() first.")
    }

    pub fn from_env() -> Result<Self> {
        let database_type =
            std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "postgres".to_string());
        let database_url = env_var("DATABASE_URL")?;
        let host = env_var("HOST")?;
        let port: u16 = env_var("PORT")?.parse()?;

        let api_server_url = std::env::var("API_SERVER_URL").unwrap_or_else(|_| {
            let host_str = if host == "0.0.0.0" {
                "localhost"
            } else {
                &host
            };
            let use_https_val = std::env::var("USE_HTTPS")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(false);
            let protocol = if use_https_val { "https" } else { "http" };
            format!("{protocol}://{host_str}:{port}")
        });

        let api_external_url =
            std::env::var("API_EXTERNAL_URL").unwrap_or_else(|_| api_server_url.clone());

        let dangerously_bypass_oauth = std::env::var("DANGEROUSLY_BYPASS_OAUTH")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let use_https = std::env::var("USE_HTTPS")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let jwt_issuer =
            std::env::var("JWT_ISSUER").unwrap_or_else(|_| "systemprompt-os".to_string());
        let jwt_secret = env_var("JWT_SECRET")?;
        let jwt_access_token_expiration = std::env::var("JWT_ACCESS_TOKEN_EXPIRATION")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(86400); // 24 hours default
        let jwt_refresh_token_expiration = std::env::var("JWT_REFRESH_TOKEN_EXPIRATION")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(2_592_000); // 30 days default

        let cargo_target_dir =
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());

        let binary_dir = std::env::var("SYSTEMPROMPT_BINARY_DIR").ok();

        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "https://modelcontextprotocol.io".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let config = Self {
            sitename: env_var("SITENAME")?,
            database_type,
            database_url,
            github_link: env_var("GITHUB_LINK")?,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            system_path: env_var("SYSTEM_PATH")?,
            host,
            port,
            admin_password: env_var("ADMIN_PASSWORD")?,
            api_server_url,
            api_external_url,
            jwt_issuer,
            jwt_secret,
            jwt_access_token_expiration,
            jwt_refresh_token_expiration,
            dangerously_bypass_oauth,
            use_https,
            cargo_target_dir,
            binary_dir,
            rate_limits: RateLimitConfig::from_env(),
            cors_allowed_origins,
        };

        config.validate_database_config()?;
        Ok(config)
    }

    pub fn validate_database_config(&self) -> Result<()> {
        let db_type = self.database_type.to_lowercase();

        if db_type != "postgres" && db_type != "postgresql" {
            return Err(anyhow::anyhow!(
                "Unsupported database type '{}'. Only 'postgres' is supported.",
                self.database_type
            ));
        }

        validate_postgres_url(&self.database_url)?;
        Ok(())
    }
}

fn env_var(name: &str) -> Result<String> {
    std::env::var(name).map_err(|e| match e {
        VarError::NotPresent => anyhow::anyhow!("Config missing: {name}"),
        VarError::NotUnicode(_) => {
            anyhow::anyhow!("Config invalid: {name} contains invalid UTF-8")
        },
    })
}

fn validate_postgres_url(url: &str) -> Result<()> {
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return Err(anyhow::anyhow!(
            "DATABASE_URL must be a PostgreSQL connection string (postgres:// or postgresql://)"
        ));
    }
    Ok(())
}

impl ConfigProvider for Config {
    fn get(&self, key: &str) -> Option<String> {
        match key {
            "database_type" => Some(self.database_type.clone()),
            "database_url" => Some(self.database_url.clone()),
            "jwt_secret" => Some(self.jwt_secret.clone()),
            "host" => Some(self.host.clone()),
            "port" => Some(self.port.to_string()),
            "system_path" => Some(self.system_path.clone()),
            "sitename" => Some(self.sitename.clone()),
            "github_link" => Some(self.github_link.clone()),
            "github_token" => self.github_token.clone(),
            "admin_password" => Some(self.admin_password.clone()),
            "api_server_url" => Some(self.api_server_url.clone()),
            "api_external_url" => Some(self.api_external_url.clone()),
            "jwt_issuer" => Some(self.jwt_issuer.clone()),
            "cargo_target_dir" => Some(self.cargo_target_dir.clone()),
            "binary_dir" => self.binary_dir.clone(),
            _ => None,
        }
    }

    fn database_url(&self) -> &str {
        &self.database_url
    }

    fn system_path(&self) -> &str {
        &self.system_path
    }

    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    fn api_port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SystemPaths;

impl SystemPaths {
    const METADATA_MCP: &'static str = "metadata/mcp";
    const SERVICES: &'static str = "crates/services";
    const SKILLS_DIR: &'static str = "crates/services/skills";
    const CONFIG_DIR: &'static str = "crates/services/config";
    const AGENTS_SUBDIR: &'static str = "crates/services/agents";
    const CORE_MIGRATIONS: &'static str = "crates/core";
    const SKILL_FILE: &'static str = "SKILL.md";
    const AGENTS_CONFIG_FILE: &'static str = "agents.yml";
    const CONFIG_FILE: &'static str = "config.yml";

    pub fn metadata_mcp(config: &Config) -> std::path::PathBuf {
        std::path::Path::new(&config.system_path).join(Self::METADATA_MCP)
    }

    pub fn services(config: &Config) -> std::path::PathBuf {
        std::env::var("SYSTEMPROMPT_SERVICES_PATH").map_or_else(
            |_| std::path::Path::new(&config.system_path).join(Self::SERVICES),
            std::path::PathBuf::from,
        )
    }

    pub fn skills(config: &Config) -> std::path::PathBuf {
        std::env::var("SYSTEMPROMPT_SKILLS_PATH").map_or_else(
            |_| std::path::Path::new(&config.system_path).join(Self::SKILLS_DIR),
            std::path::PathBuf::from,
        )
    }

    pub fn config_dir(config: &Config) -> std::path::PathBuf {
        std::path::Path::new(&config.system_path).join(Self::CONFIG_DIR)
    }

    pub fn agents_config(config: &Config) -> std::path::PathBuf {
        std::env::var("SYSTEMPROMPT_CONFIG_PATH").map_or_else(
            |_| {
                std::path::Path::new(&config.system_path)
                    .join(Self::AGENTS_SUBDIR)
                    .join(Self::AGENTS_CONFIG_FILE)
            },
            std::path::PathBuf::from,
        )
    }

    pub fn services_config(config: &Config) -> std::path::PathBuf {
        std::path::Path::new(&config.system_path)
            .join(Self::CONFIG_DIR)
            .join(Self::CONFIG_FILE)
    }

    pub fn core_migrations(config: &Config, module_name: &str) -> std::path::PathBuf {
        std::path::Path::new(&config.system_path)
            .join(Self::CORE_MIGRATIONS)
            .join(module_name)
            .join("migrations")
    }

    pub const fn skill_file() -> &'static str {
        Self::SKILL_FILE
    }

    pub const fn agents_config_file() -> &'static str {
        Self::AGENTS_CONFIG_FILE
    }

    pub const fn config_file() -> &'static str {
        Self::CONFIG_FILE
    }

    pub fn resolve_mcp_server(config: &Config, server_name: &str) -> std::path::PathBuf {
        Self::services(config).join(server_name)
    }

    pub fn resolve_skill(config: &Config, skill_name: &str) -> std::path::PathBuf {
        Self::skills(config).join(skill_name)
    }

    pub fn content_config(_config: &Config) -> std::path::PathBuf {
        if let Ok(path) = std::env::var("CONTENT_CONFIG_PATH") {
            std::path::PathBuf::from(path)
        } else {
            eprintln!("ERROR: CONTENT_CONFIG_PATH environment variable is required");
            panic!("Missing required environment variable: CONTENT_CONFIG_PATH");
        }
    }
}
