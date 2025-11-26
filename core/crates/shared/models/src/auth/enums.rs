use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JwtAudience {
    Web,
    Api,
    A2a,
    Mcp,
    Internal,
}

impl JwtAudience {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Api => "api",
            Self::A2a => "a2a",
            Self::Mcp => "mcp",
            Self::Internal => "internal",
        }
    }

    pub fn standard() -> Vec<Self> {
        vec![Self::Web, Self::Api, Self::A2a, Self::Mcp]
    }

    pub fn service() -> Vec<Self> {
        vec![Self::Api, Self::Mcp, Self::A2a, Self::Internal]
    }
}

impl fmt::Display for JwtAudience {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for JwtAudience {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "web" => Ok(Self::Web),
            "api" => Ok(Self::Api),
            "a2a" => Ok(Self::A2a),
            "mcp" => Ok(Self::Mcp),
            "internal" => Ok(Self::Internal),
            _ => Err(anyhow!("Invalid JWT audience: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    Admin,
    Standard,
    Anon,
    Service,
    Unknown,
}

impl UserType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Standard => "standard",
            Self::Anon => "anon",
            Self::Service => "service",
            Self::Unknown => "unknown",
        }
    }

    pub const fn rate_tier(&self) -> RateLimitTier {
        match self {
            Self::Admin => RateLimitTier::Admin,
            Self::Standard => RateLimitTier::Standard,
            Self::Service => RateLimitTier::Service,
            Self::Anon | Self::Unknown => RateLimitTier::Anon,
        }
    }
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "admin" => Ok(Self::Admin),
            "standard" => Ok(Self::Standard),
            "anon" => Ok(Self::Anon),
            "service" => Ok(Self::Service),
            "unknown" => Ok(Self::Unknown),
            _ => Err(anyhow!("Invalid user type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    Bearer,
}

impl TokenType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bearer => "Bearer",
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bearer")
    }
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Bearer
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitTier {
    Admin,
    Standard,
    Anon,
    Service,
}

impl RateLimitTier {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Standard => "standard",
            Self::Anon => "anon",
            Self::Service => "service",
        }
    }
}

impl fmt::Display for RateLimitTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RateLimitTier {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "admin" => Ok(Self::Admin),
            "standard" => Ok(Self::Standard),
            "anon" => Ok(Self::Anon),
            "service" => Ok(Self::Service),
            _ => Err(anyhow!("Invalid rate limit tier: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Anonymous,
}

impl UserRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
            Self::Anonymous => "anonymous",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserRole {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            "anonymous" | "anon" => Ok(Self::Anonymous),
            _ => Err(anyhow!("Invalid user role: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Temporary,
    Inactive,
    Suspended,
}

impl UserStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Temporary => "temporary",
            Self::Inactive => "inactive",
            Self::Suspended => "suspended",
        }
    }

    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Self::Active),
            "temporary" => Ok(Self::Temporary),
            "inactive" => Ok(Self::Inactive),
            "suspended" => Ok(Self::Suspended),
            _ => Err(anyhow!("Invalid user status: {s}")),
        }
    }
}
