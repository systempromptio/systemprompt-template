use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub tenant_id: String,
    pub api_url: String,
    pub api_token: String,
    pub services_path: String,
    pub database_url: Option<String>,
}

impl SyncConfig {
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::credentials_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let creds: serde_json::Value = serde_json::from_str(&content)?;

                return Ok(Self {
                    tenant_id: creds["tenant_id"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    api_url: creds["api_url"]
                        .as_str()
                        .unwrap_or("https://api.systemprompt.io")
                        .to_string(),
                    api_token: creds["api_token"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    services_path: env::var("SYSTEMPROMPT_SERVICES_PATH")
                        .unwrap_or_else(|_| "services".to_string()),
                    database_url: env::var("DATABASE_URL").ok(),
                });
            }
        }

        Ok(Self {
            tenant_id: env::var("SYSTEMPROMPT_TENANT_ID").unwrap_or_default(),
            api_url: env::var("SYSTEMPROMPT_API_URL")
                .unwrap_or_else(|_| "https://api.systemprompt.io".to_string()),
            api_token: env::var("SYSTEMPROMPT_API_TOKEN").unwrap_or_default(),
            services_path: env::var("SYSTEMPROMPT_SERVICES_PATH")
                .unwrap_or_else(|_| "services".to_string()),
            database_url: env::var("DATABASE_URL").ok(),
        })
    }

    fn credentials_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".systemprompt/credentials.json"))
    }

    pub fn is_configured(&self) -> bool {
        !self.tenant_id.is_empty() && !self.api_token.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SyncDirection {
    Push,
    Pull,
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
            Self::Pull => write!(f, "pull"),
        }
    }
}

impl std::str::FromStr for SyncDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "push" => Ok(Self::Push),
            "pull" => Ok(Self::Pull),
            _ => anyhow::bail!("Invalid sync direction: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_direction_from_str() {
        assert_eq!(
            "push".parse::<SyncDirection>().unwrap(),
            SyncDirection::Push
        );
        assert_eq!(
            "pull".parse::<SyncDirection>().unwrap(),
            SyncDirection::Pull
        );
        assert_eq!(
            "PUSH".parse::<SyncDirection>().unwrap(),
            SyncDirection::Push
        );
        assert!("invalid".parse::<SyncDirection>().is_err());
    }

    #[test]
    fn sync_direction_display() {
        assert_eq!(SyncDirection::Push.to_string(), "push");
        assert_eq!(SyncDirection::Pull.to_string(), "pull");
    }

    #[test]
    fn sync_config_is_configured() {
        let config = SyncConfig {
            tenant_id: "test".to_string(),
            api_url: "https://api.systemprompt.io".to_string(),
            api_token: "token".to_string(),
            services_path: "services".to_string(),
            database_url: None,
        };
        assert!(config.is_configured());

        let empty_config = SyncConfig {
            tenant_id: String::new(),
            api_url: String::new(),
            api_token: String::new(),
            services_path: String::new(),
            database_url: None,
        };
        assert!(!empty_config.is_configured());
    }
}
