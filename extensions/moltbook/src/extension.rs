use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;

pub const SCHEMA_MOLTBOOK_AGENTS: &str = include_str!("../schema/001_moltbook_agents.sql");
pub const SCHEMA_MOLTBOOK_POSTS: &str = include_str!("../schema/002_moltbook_posts.sql");
pub const SCHEMA_MOLTBOOK_ANALYTICS: &str = include_str!("../schema/003_moltbook_analytics.sql");

#[derive(Debug, Default, Clone)]
pub struct MoltbookExtension;

impl MoltbookExtension {
    pub const PREFIX: &'static str = "moltbook";

    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub const fn base_path() -> &'static str {
        "/api/v1/moltbook"
    }
}

impl Extension for MoltbookExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "moltbook",
            name: "Moltbook - AI Agent Social Network Integration",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("moltbook_agents", SCHEMA_MOLTBOOK_AGENTS),
            SchemaDefinition::inline("moltbook_posts", SCHEMA_MOLTBOOK_POSTS),
            SchemaDefinition::inline("moltbook_analytics", SCHEMA_MOLTBOOK_ANALYTICS),
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let pool = db.pool()?;

        let router = crate::api::router(pool);
        Some(ExtensionRouter::new(router, Self::base_path()))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![
            Arc::new(crate::jobs::MoltbookSyncJob),
            Arc::new(crate::jobs::MoltbookAnalyticsJob),
        ]
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

register_extension!(MoltbookExtension);
