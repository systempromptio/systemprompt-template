use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::sync::SyncDirection as CoreSyncDirection;

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
            _ => anyhow::bail!("Invalid sync direction: {s}"),
        }
    }
}

impl SyncDirection {
    #[must_use]
    pub const fn to_core(self) -> CoreSyncDirection {
        match self {
            Self::Push => CoreSyncDirection::Push,
            Self::Pull => CoreSyncDirection::Pull,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTarget {
    Files,
    Database,
    Content,
    Skills,
    All,
}

impl std::str::FromStr for SyncTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "files" => Ok(Self::Files),
            "database" => Ok(Self::Database),
            "content" => Ok(Self::Content),
            "skills" => Ok(Self::Skills),
            "all" => Ok(Self::All),
            _ => anyhow::bail!("Invalid sync target: {s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportTarget {
    Content,
    Skills,
    All,
}

impl std::str::FromStr for ExportTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "content" => Ok(Self::Content),
            "skills" => Ok(Self::Skills),
            "all" => Ok(Self::All),
            _ => anyhow::bail!("Invalid export target: {s}"),
        }
    }
}
