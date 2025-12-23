use anyhow::{Context, Result};
use std::time::Instant;
use systemprompt::credentials::CredentialsBootstrap;
use systemprompt::database::DbPool;
use systemprompt::models::Config;
use systemprompt::sync::{
    ContentLocalSync, LocalSyncDirection, SkillsLocalSync, SyncConfig as CoreSyncConfig,
    SyncDirection as CoreSyncDirection, SyncService as CoreSyncService,
};

use super::deploy::deploy_crate;
use super::types::SyncDirection;
use super::{
    CloudStatus, DatabaseSyncSummary, DeployCrateResult, SyncAllResult, SyncDatabaseResult,
    SyncFilesResult, SyncStatusResult, SyncSummary, SyncTable, TableSyncResult,
};

#[derive(Debug)]
pub struct SyncService {
    db_pool: DbPool,
}

impl SyncService {
    #[must_use]
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    fn build_core_config(direction: CoreSyncDirection, dry_run: bool) -> Result<CoreSyncConfig> {
        let config = Config::global();
        let creds =
            CredentialsBootstrap::require().context("Cloud credentials required for sync")?;

        let tenant_id = creds.tenant_id.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No tenant configured. Run 'systemprompt cloud setup'")
        })?;

        let mut builder = CoreSyncConfig::builder(
            tenant_id,
            &creds.api_url,
            &creds.api_token,
            &config.services_path,
        )
        .with_direction(direction)
        .with_dry_run(dry_run);

        if !config.database_url.is_empty() {
            builder = builder.with_database_url(&config.database_url);
        }

        Ok(builder.build())
    }

    pub async fn sync_files(
        &self,
        direction: SyncDirection,
        dry_run: bool,
    ) -> Result<SyncFilesResult> {
        let start = Instant::now();
        let core_config = Self::build_core_config(direction.to_core(), dry_run)?;
        let core_service = CoreSyncService::new(core_config);

        let result = core_service.sync_files().await?;

        let summary = SyncSummary {
            total_files: result.items_synced + result.items_skipped,
            created: if result.success {
                result.items_synced
            } else {
                0
            },
            updated: 0,
            deleted: 0,
            unchanged: result.items_skipped,
            skipped: result.items_skipped,
            duration_ms: start.elapsed().as_millis() as u64,
        };

        Ok(SyncFilesResult {
            direction,
            dry_run,
            files_synced: Vec::new(),
            files_skipped: Vec::new(),
            summary,
        })
    }

    pub async fn sync_database(
        &self,
        direction: SyncDirection,
        dry_run: bool,
        _tables: Option<Vec<SyncTable>>,
    ) -> Result<SyncDatabaseResult> {
        let start = Instant::now();
        let core_config = Self::build_core_config(direction.to_core(), dry_run)?;
        let core_service = CoreSyncService::new(core_config);

        let result = core_service.sync_database().await?;

        let (agents, skills, contexts) = result.details.as_ref().map_or((0, 0, 0), |details| {
            (
                details["agents"].as_u64().map_or(0, |v| v as usize),
                details["skills"].as_u64().map_or(0, |v| v as usize),
                details["contexts"].as_u64().map_or(0, |v| v as usize),
            )
        });

        let tables_synced = vec![
            TableSyncResult {
                table_name: "agents".to_string(),
                records_synced: agents,
                records_created: agents,
                records_updated: 0,
                records_deleted: 0,
            },
            TableSyncResult {
                table_name: "skills".to_string(),
                records_synced: skills,
                records_created: skills,
                records_updated: 0,
                records_deleted: 0,
            },
            TableSyncResult {
                table_name: "contexts".to_string(),
                records_synced: contexts,
                records_created: contexts,
                records_updated: 0,
                records_deleted: 0,
            },
        ];

        let summary = DatabaseSyncSummary {
            total_tables: 3,
            total_records_synced: result.items_synced,
            duration_ms: start.elapsed().as_millis() as u64,
        };

        Ok(SyncDatabaseResult {
            direction,
            dry_run,
            tables_synced,
            summary,
        })
    }

    #[allow(clippy::unused_async)]
    pub async fn sync_content(
        &self,
        direction: SyncDirection,
        dry_run: bool,
        _filter: Option<String>,
    ) -> Result<SyncFilesResult> {
        let start = Instant::now();
        let content_sync = ContentLocalSync::new(DbPool::clone(&self.db_pool));

        let local_direction = match direction {
            SyncDirection::Push => LocalSyncDirection::ToDisk,
            SyncDirection::Pull => LocalSyncDirection::ToDatabase,
        };

        let summary = SyncSummary {
            total_files: 0,
            created: 0,
            updated: 0,
            deleted: 0,
            unchanged: 0,
            skipped: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        };

        tracing::info!(
            direction = ?local_direction,
            dry_run = dry_run,
            "Content sync - implementation pending content source configuration"
        );

        let _ = content_sync;

        Ok(SyncFilesResult {
            direction,
            dry_run,
            files_synced: Vec::new(),
            files_skipped: Vec::new(),
            summary,
        })
    }

    #[allow(clippy::unused_async)]
    pub async fn sync_skills(
        &self,
        direction: SyncDirection,
        dry_run: bool,
        _filter: Option<String>,
    ) -> Result<SyncFilesResult> {
        let start = Instant::now();
        let config = Config::global();
        let skills_path = std::path::PathBuf::from(&config.skills_path);
        let _skills_sync = SkillsLocalSync::new(DbPool::clone(&self.db_pool), skills_path);

        let local_direction = match direction {
            SyncDirection::Push => LocalSyncDirection::ToDisk,
            SyncDirection::Pull => LocalSyncDirection::ToDatabase,
        };

        let summary = SyncSummary {
            total_files: 0,
            created: 0,
            updated: 0,
            deleted: 0,
            unchanged: 0,
            skipped: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        };

        tracing::info!(
            direction = ?local_direction,
            dry_run = dry_run,
            "Skills sync - implementation pending skills configuration"
        );

        Ok(SyncFilesResult {
            direction,
            dry_run,
            files_synced: Vec::new(),
            files_skipped: Vec::new(),
            summary,
        })
    }

    pub async fn deploy_crate(
        &self,
        skip_build: bool,
        tag: Option<String>,
    ) -> Result<DeployCrateResult> {
        deploy_crate(skip_build, tag, Self::build_core_config).await
    }

    pub async fn sync_all(&self, direction: SyncDirection, dry_run: bool) -> Result<SyncAllResult> {
        let start = Instant::now();

        let files_result = self.sync_files(direction, dry_run).await.ok();
        let database_result = self.sync_database(direction, dry_run, None).await.ok();

        let deploy_result = if direction == SyncDirection::Push && !dry_run {
            self.deploy_crate(false, None).await.ok()
        } else {
            None
        };

        Ok(SyncAllResult {
            direction,
            dry_run,
            files_result,
            database_result,
            deploy_result,
            total_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    #[allow(clippy::unused_async)]
    pub async fn get_status(&self) -> Result<SyncStatusResult> {
        let config = Config::global();

        let creds = CredentialsBootstrap::get().ok().flatten();
        let is_configured = creds
            .as_ref()
            .is_some_and(|c| c.tenant_id.is_some() && !c.api_token.is_empty());

        let cloud_status = CloudStatus {
            connected: is_configured,
            deployment_status: if is_configured {
                Some("running".to_string())
            } else {
                None
            },
            last_deployment: None,
            app_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        let tenant_id = creds
            .as_ref()
            .and_then(|c| c.tenant_id.clone())
            .map_or_else(String::new, |t| t);

        let api_url = creds
            .as_ref()
            .map_or_else(|| config.api_server_url.clone(), |c| c.api_url.clone());

        Ok(SyncStatusResult {
            tenant_id,
            api_url,
            services_path: config.services_path.clone(),
            database_configured: !config.database_url.is_empty(),
            cloud_status,
            last_sync: None,
        })
    }
}
