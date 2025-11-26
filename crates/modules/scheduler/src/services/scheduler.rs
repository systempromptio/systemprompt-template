use crate::models::SchedulerConfig;
use crate::repository::SchedulerRepository;
use crate::services::jobs;
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogContext, LogService};
use systemprompt_core_system::AppContext;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Debug)]
pub struct SchedulerService {
    config: SchedulerConfig,
    db_pool: DbPool,
    repository: SchedulerRepository,
    app_context: Arc<AppContext>,
}

impl SchedulerService {
    #[must_use]
    pub fn new(config: SchedulerConfig, db_pool: DbPool, app_context: Arc<AppContext>) -> Self {
        let repository = SchedulerRepository::new(db_pool.clone());
        Self {
            config,
            db_pool,
            repository,
            app_context,
        }
    }

    pub async fn start(self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Scheduler is disabled");
            return Ok(());
        }

        tracing::info!("Starting scheduler with {} jobs", self.config.jobs.len());

        let scheduler = JobScheduler::new().await?;

        for job_config in &self.config.jobs {
            if !job_config.enabled {
                tracing::info!("Skipping disabled job: {}", job_config.name);
                continue;
            }

            self.repository
                .upsert_job(&job_config.name, &job_config.schedule, job_config.enabled)
                .await?;

            let job = self.create_job(job_config).await?;
            scheduler.add(job).await?;

            tracing::info!(
                "Registered job '{}' with schedule '{}'",
                job_config.name,
                job_config.schedule
            );
        }

        scheduler.start().await?;

        tracing::info!("Scheduler started successfully");

        Ok(())
    }

    async fn create_job(&self, job_config: &crate::models::JobConfig) -> Result<Job> {
        let job_name = job_config.name.clone();
        let schedule = job_config.schedule.clone();
        let db_pool = self.db_pool.clone();
        let repository = self.repository.clone();
        let app_context = self.app_context.clone();

        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let job_name = job_name.clone();
            let db_pool = db_pool.clone();
            let repository = repository.clone();
            let app_context = app_context.clone();

            Box::pin(async move {
                let log_context = LogContext::new()
                    .with_session_id("scheduler")
                    .with_trace_id(&format!("scheduler-{}", uuid::Uuid::new_v4()))
                    .with_user_id("system");

                let logger = LogService::new(db_pool.clone(), log_context);

                logger
                    .info("scheduler", &format!("Starting job: {job_name}"))
                    .await
                    .ok();

                if let Err(e) = repository.increment_run_count(&job_name).await {
                    logger
                        .error(
                            "scheduler",
                            &format!("Failed to increment run count for {}: {}", job_name, e),
                        )
                        .await
                        .ok();
                }

                let result = match job_name.as_str() {
                    "cleanup_anonymous_users" => {
                        jobs::cleanup_anonymous_users(
                            db_pool.clone(),
                            logger.clone(),
                            app_context.clone(),
                        )
                        .await
                    },
                    "cleanup_inactive_sessions" => {
                        jobs::cleanup_inactive_sessions(
                            db_pool.clone(),
                            logger.clone(),
                            app_context.clone(),
                        )
                        .await
                    },
                    "database_cleanup" => {
                        jobs::database_cleanup(db_pool.clone(), logger.clone(), app_context.clone())
                            .await
                    },
                    "regenerate_static_content" => {
                        jobs::regenerate_static_content(
                            db_pool.clone(),
                            logger.clone(),
                            app_context.clone(),
                        )
                        .await
                    },
                    "ingest_content" => {
                        jobs::ingest_content(db_pool.clone(), logger.clone(), app_context.clone())
                            .await
                    },
                    "evaluate_conversations" => {
                        jobs::evaluate_conversations(
                            db_pool.clone(),
                            logger.clone(),
                            app_context.clone(),
                        )
                        .await
                    },
                    _ => {
                        logger
                            .error("scheduler", &format!("Unknown job: {}", job_name))
                            .await
                            .ok();
                        Err(anyhow::anyhow!("Unknown job: {}", job_name))
                    },
                };

                match result {
                    Ok(_) => {
                        if let Err(e) = repository
                            .update_job_execution(&job_name, "success", None, None)
                            .await
                        {
                            logger
                                .error(
                                    "scheduler",
                                    &format!(
                                        "Failed to update job execution status for {}: {}",
                                        job_name, e
                                    ),
                                )
                                .await
                                .ok();
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("{}", e);
                        logger.error("scheduler", &error_msg).await.ok();
                        if let Err(e) = repository
                            .update_job_execution(&job_name, "failed", Some(&error_msg), None)
                            .await
                        {
                            logger
                                .error(
                                    "scheduler",
                                    &format!(
                                        "Failed to update failed job status for {}: {}",
                                        job_name, e
                                    ),
                                )
                                .await
                                .ok();
                        }
                    },
                }
            })
        })?;

        Ok(job)
    }
}
