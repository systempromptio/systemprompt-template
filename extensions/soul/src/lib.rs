#![allow(clippy::needless_raw_string_hashes)]

pub mod discord;
pub mod error;
pub mod extension;
pub mod jobs;
pub mod models;
pub mod repository;
pub mod services;

pub use discord::{DiscordConfig, DiscordConfigValidated, DiscordHandler, DiscordService, GatewayConfig};
pub use error::SoulError;
pub use extension::SoulExtension;
pub use jobs::{DiscordGatewayJob, HeartbeatJob, MemorySynthesisJob};
pub use models::{CreateMemoryParams, MemoryCategory, MemoryType, SoulMemory};
pub use repository::MemoryRepository;
pub use services::MemoryService;
