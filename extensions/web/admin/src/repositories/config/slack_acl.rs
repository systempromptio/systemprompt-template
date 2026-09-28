//! Inbound Slack apps: each app's `authz.allowed_roles` projected onto its
//! `slack_workspace:<workspace_id>` entity.
//!
//! The one entitlement `rules.yaml` does not declare. The gate stays with the
//! app it gates (`services/slack/*.yaml`), core writes the rows, and the
//! entity is closed to every role the app does not list. It runs at every
//! boot after the sync planes have been reconciled, and the access-control
//! plane leaves these entity kinds out of its drift
//! ([`crate::repositories::access_control::rules::EXTERNALLY_PROJECTED_KINDS`])
//! so an overwrite from `rules.yaml` never deletes a Slack gate.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::{AccessControlIngestionService, IngestOptions};
use systemprompt_web_shared::error::MarketplaceError;

// Why: an unreadable services config is not this loader's failure to report —
// the server does not start without one, and treating it as fatal here would
// turn every unrelated config error into "access control failed to load".
pub async fn load_slack_apps(pool: &PgPool) -> Result<usize, MarketplaceError> {
    let Ok(services) = ConfigLoader::load() else {
        return Ok(0);
    };
    if services.slack_apps.is_empty() {
        return Ok(0);
    }

    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_slack_apps(
            &services.slack_apps,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
                ..IngestOptions::default()
            },
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    Ok(ingested.inserted + ingested.updated)
}
