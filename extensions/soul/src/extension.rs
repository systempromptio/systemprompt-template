use std::sync::Arc;

use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;

use crate::jobs::{HeartbeatJob, MemorySynthesisJob};

pub const SCHEMA_SOUL_MEMORIES: &str = include_str!("../schema/001_soul_memories.sql");

#[derive(Debug, Default, Clone)]
pub struct SoulExtension;

impl SoulExtension {
    pub const PREFIX: &'static str = "soul";

    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Extension for SoulExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "soul",
            name: "Soul - Memory System & Proactive Updates",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![SchemaDefinition::inline(
            "soul_memories",
            SCHEMA_SOUL_MEMORIES,
        )]
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(MemorySynthesisJob), Arc::new(HeartbeatJob)]
    }

    fn priority(&self) -> u32 {
        200
    }

    fn migration_weight(&self) -> u32 {
        200
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }
}

register_extension!(SoulExtension);
