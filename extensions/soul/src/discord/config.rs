use std::path::Path;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use systemprompt::models::SecretsBootstrap;

const MIN_SNOWFLAKE_LENGTH: usize = 17;
const MAX_SNOWFLAKE_LENGTH: usize = 20;

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    pub default_channel_id: Option<String>,
    pub default_user_id: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub gateway: Option<GatewayConfigRaw>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfigRaw {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_target_agent")]
    pub target_agent: String,
    #[serde(default = "default_message_prefix")]
    pub message_prefix: String,
    #[serde(default)]
    pub ignore_channels: Vec<String>,
    #[serde(default)]
    pub ignore_bots: bool,
}

fn default_true() -> bool {
    true
}

fn default_target_agent() -> String {
    "systemprompt_hub".to_string()
}

fn default_message_prefix() -> String {
    "DISCORD_MESSAGE".to_string()
}

impl Default for GatewayConfigRaw {
    fn default() -> Self {
        Self {
            enabled: true,
            target_agent: default_target_agent(),
            message_prefix: default_message_prefix(),
            ignore_channels: Vec::new(),
            ignore_bots: true,
        }
    }
}

#[derive(Clone)]
pub struct DiscordConfigValidated {
    bot_token: SecretString,
    default_channel_id: Option<String>,
    default_user_id: Option<String>,
    enabled: bool,
    gateway: GatewayConfig,
}

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub enabled: bool,
    pub target_agent: String,
    pub message_prefix: String,
    pub ignore_channels: Vec<String>,
    pub ignore_bots: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_agent: default_target_agent(),
            message_prefix: default_message_prefix(),
            ignore_channels: Vec::new(),
            ignore_bots: true,
        }
    }
}

impl std::fmt::Debug for DiscordConfigValidated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordConfigValidated")
            .field("bot_token", &"[REDACTED]")
            .field("default_channel_id", &self.default_channel_id)
            .field("default_user_id", &self.default_user_id)
            .field("enabled", &self.enabled)
            .field("gateway", &self.gateway)
            .finish()
    }
}

impl DiscordConfigValidated {
    fn get_bot_token_from_secrets() -> anyhow::Result<String> {
        let secrets =
            SecretsBootstrap::get().map_err(|e| anyhow::anyhow!("Failed to get secrets: {}", e))?;

        secrets.get("discord_bot_token").cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "discord_bot_token not found in secrets. Add it to your profile's secrets.json"
            )
        })
    }

    pub fn from_raw(raw: DiscordConfig) -> anyhow::Result<Self> {
        let bot_token = Self::get_bot_token_from_secrets()?;

        if bot_token.trim().is_empty() {
            anyhow::bail!("Discord bot token cannot be empty");
        }

        if raw.enabled && raw.default_channel_id.is_none() && raw.default_user_id.is_none() {
            anyhow::bail!(
                "At least one default target (channel or user) should be configured when enabled"
            );
        }

        if let Some(ref id) = raw.default_channel_id {
            if !Self::is_valid_snowflake(id) {
                anyhow::bail!("Invalid Discord channel ID format: {}", id);
            }
        }

        if let Some(ref id) = raw.default_user_id {
            if !Self::is_valid_snowflake(id) {
                anyhow::bail!("Invalid Discord user ID format: {}", id);
            }
        }

        let gateway = raw
            .gateway
            .map_or_else(GatewayConfig::default, |g| GatewayConfig {
                enabled: g.enabled,
                target_agent: g.target_agent,
                message_prefix: g.message_prefix,
                ignore_channels: g.ignore_channels,
                ignore_bots: g.ignore_bots,
            });

        Ok(Self {
            bot_token: SecretString::from(bot_token),
            default_channel_id: raw.default_channel_id,
            default_user_id: raw.default_user_id,
            enabled: raw.enabled,
            gateway,
        })
    }

    fn is_valid_snowflake(id: &str) -> bool {
        id.len() >= MIN_SNOWFLAKE_LENGTH
            && id.len() <= MAX_SNOWFLAKE_LENGTH
            && id.chars().all(|c| c.is_ascii_digit())
    }

    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;

        let raw: DiscordConfig = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config YAML: {}", e))?;

        Self::from_raw(raw)
    }

    pub fn load_from_default_paths() -> anyhow::Result<Self> {
        if let Ok(custom_path) = std::env::var("SOUL_DISCORD_CONFIG") {
            return Self::load_from_file(Path::new(&custom_path));
        }

        let paths = [
            "./services/config/discord.yaml",
            "./services/config/extensions/discord.yaml",
        ];

        for path in &paths {
            let path = Path::new(path);
            if path.exists() {
                return Self::load_from_file(path);
            }
        }

        anyhow::bail!(
            "Discord config not found. Create services/config/discord.yaml with:\n\
            default_user_id: \"YOUR_USER_ID\"\n\
            enabled: true\n\n\
            And add discord_bot_token to your profile's secrets.json\n\
            Or set SOUL_DISCORD_CONFIG environment variable to custom config path."
        )
    }

    pub fn bot_token(&self) -> &str {
        self.bot_token.expose_secret()
    }

    pub fn default_channel_id(&self) -> Option<&str> {
        self.default_channel_id.as_deref()
    }

    pub fn default_user_id(&self) -> Option<&str> {
        self.default_user_id.as_deref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn gateway(&self) -> &GatewayConfig {
        &self.gateway
    }
}
