//! `conversation_judge` job: the one AI label on top of the deterministic
//! conversation record.
//!
//! Each tick: queue every quiet or closed conversation with at least one turn
//! that has not been judged (or grew since it was), lease a batch, read each
//! transcript through the same readers the conversation page uses, and ask
//! the configured model — in one structured call — for a title, a summary,
//! the intent category, an outcome and a single 0–100 `completion` score:
//! did the assistant deliver what the person originally asked for. The
//! verdict lands in `conversation_analyses` and every Analysis page joins it.
//!
//! Judge calls go through the process's shared `AiService` as this job's
//! actor, so they are audited like any other request but are excluded from
//! every conversation view (`27_conversation_requests.sql`). A daily cost cap
//! stops the run — leases are handed back — rather than letting a backlog
//! burn budget.
//!
//! Two gates: the scheduler entry's `enabled:` and the profile's
//! `judge.automatic`, which switches off every background judge spend at
//! once. A manual `-p context_id=<id>` run bypasses the latter.

mod classifier;
mod discovery;
mod judge;
mod params;
mod repository;
mod transcript;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::database::DbPool;
use systemprompt::identifiers::Actor;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::analytics::context_detail::{
    list_context_requests, list_messages_for_requests,
};
use systemprompt_web_admin::repositories::analytics::context_tool_calls::list_tool_calls_for_requests;
use systemprompt_web_admin::test_support::{
    TranscriptOptions, build_conversation, transcript_request_ids,
};

use crate::JobError;

pub use classifier::{ConversationClassifier, JudgeVerdict};
pub use judge::{Category, Classification, Outcome, classification_schema, parse_classification};
#[doc(hidden)]
pub use params::JudgeParams;
pub use transcript::{TranscriptMeta, render_transcript};

use discovery::DiscoveryWindow;
use repository::{AnalysisLease, Completion, ConversationJudgeRepository};

pub(crate) const JOB_NAME: &str = "conversation_judge";

#[derive(Debug, Clone, Copy, Default)]
pub struct ConversationJudgeJob;

struct Tick<'a> {
    pool: &'a PgPool,
    repo: ConversationJudgeRepository,
    classifier: &'a dyn ConversationClassifier,
    params: JudgeParams,
    // Why: the profile's automatic switch is off — no discovery, and only
    // rows a person queued from the console are judged.
    manual_only: bool,
}

// Why: the scheduler and DB-backed tests use this one control-flow runner;
// only the inference boundary is supplied by the caller.
#[doc(hidden)]
pub async fn run_with_classifier(
    pool: &PgPool,
    params: JudgeParams,
    manual_only: bool,
    classifier: &dyn ConversationClassifier,
) -> Result<JobResult, JobError> {
    let tick = Tick {
        pool,
        repo: ConversationJudgeRepository::new(pool.clone()),
        classifier,
        params,
        manual_only,
    };
    ConversationJudgeJob::run_tick(&tick).await
}

enum Step {
    Judged,
    Failed,
    CapReached,
}

impl ConversationJudgeJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        let repo = &tick.repo;
        repo.update_judge_costs().await?;
        if let Some(context_id) = &tick.params.context_id {
            if !repo.mark_for_rejudging(context_id).await? {
                return Ok(JobResult::success()
                    .with_message(format!("no gateway requests for context {context_id}")));
            }
        } else if tick.manual_only {
            tracing::debug!("conversation judge: automatic off, manual requests only");
        } else {
            let queued = repo
                .discover_and_enqueue(DiscoveryWindow {
                    quiet_minutes: tick.params.quiet_minutes,
                    lookback_days: tick.params.lookback_days,
                })
                .await?;
            tracing::debug!(queued, "conversation judge discovery");
        }

        let batch = if tick.params.context_id.is_some() {
            1
        } else {
            tick.params.batch_size
        };
        let leases = repo
            .claim(batch, tick.manual_only, tick.params.context_id.as_ref())
            .await?;
        let (mut judged, mut failed) = (0u64, 0u64);
        for (index, lease) in leases.iter().enumerate() {
            match Self::judge_one(tick, lease).await {
                Ok(Step::Judged) => judged += 1,
                Ok(Step::Failed) => failed += 1,
                Ok(Step::CapReached) => {
                    repo.release(&leases[index..]).await?;
                    tracing::warn!(
                        cap_microdollars = tick.params.daily_cost_cap_microdollars,
                        "conversation judge daily cost cap reached; leases released"
                    );
                    break;
                },
                Err(error) => {
                    failed += 1;
                    repo.retry(lease, &error.to_string()).await?;
                    tracing::warn!(%error, context_id = %lease.context_id, "conversation judgement will retry");
                },
            }
        }
        Ok(JobResult::success().with_stats(judged, failed))
    }

    async fn judge_one(tick: &Tick<'_>, lease: &AnalysisLease) -> Result<Step, JobError> {
        let repo = &tick.repo;
        let spend = repo.get_job_spend_today(JOB_NAME).await?;
        if spend >= tick.params.daily_cost_cap_microdollars {
            return Ok(Step::CapReached);
        }
        let Some(meta) = repo.find_source_meta(&lease.context_id).await? else {
            repo.retry(lease, "conversation has no readable requests")
                .await?;
            return Ok(Step::Failed);
        };

        let requests = list_context_requests(tick.pool, &lease.context_id).await?;
        let ids = transcript_request_ids(&requests);
        let messages = list_messages_for_requests(tick.pool, &ids.messages).await?;
        let tool_calls = list_tool_calls_for_requests(tick.pool, &ids.tool_calls).await?;
        let view = build_conversation(
            &messages,
            &tool_calls,
            &requests,
            TranscriptOptions::owner_facing(),
        );

        let (classification, ai_request_id, input_tokens, output_tokens) = if view.has_content {
            let hooked = repo.list_hooked_skills(&lease.context_id).await?;
            let transcript_meta = TranscriptMeta {
                client_kind: meta.client_kind.clone(),
                model: meta.model.clone(),
                turn_count: meta.turn_count,
                tool_call_count: meta.tool_call_count,
                error_count: meta.error_count,
                duration_minutes: match (meta.first_at, meta.last_at) {
                    (Some(first), Some(last)) => (last - first).num_minutes().max(0),
                    _ => 0,
                },
                hooked_skills: hooked,
            };
            let transcript =
                render_transcript(&transcript_meta, &view, tick.params.transcript_token_budget);
            let verdict = tick.classifier.classify(&transcript).await?;
            (
                verdict.classification,
                Some(verdict.ai_request_id),
                verdict.input_tokens,
                verdict.output_tokens,
            )
        } else {
            (Classification::unreadable(), None, None, None)
        };

        repo.complete(
            lease,
            Completion {
                classification: &classification,
                provider: &tick.params.provider,
                model: &tick.params.model,
                ai_request_id: ai_request_id.as_deref(),
                input_tokens,
                output_tokens,
                trigger: if tick.params.context_id.is_some() || lease.manual {
                    "manual"
                } else {
                    "automatic"
                },
                fingerprint: meta.fingerprint,
            },
        )
        .await?;
        Ok(Step::Judged)
    }
}

#[async_trait::async_trait]
impl Job for ConversationJudgeJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Labels each finished conversation with a title, summary, intent and one completion score"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn schedule(&self) -> &'static str {
        "0 */5 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let app = ctx
            .app_context::<Arc<AppContext>>()
            .ok_or(JobError::MissingContext("AppContext"))?;
        let params = JudgeParams::from_context(ctx)?;
        let automatic = ProfileBootstrap::get()
            .map_err(JobError::from)?
            .judge
            .automatic;
        let Some(ai) = app.ai_service_arc() else {
            return Ok(
                JobResult::success().with_message("no AI provider is configured; judge idle")
            );
        };
        let judge = judge::Judge {
            ai,
            actor: Actor::job(ctx.actor().user_id.clone(), JOB_NAME),
            provider: params.provider.clone(),
            model: params.model.clone(),
            max_output_tokens: params.max_output_tokens,
        };
        let pool = db.write_pool();
        let manual_only = !automatic && params.context_id.is_none();
        Ok(run_with_classifier(&pool, params, manual_only, &judge).await?)
    }
}

systemprompt::traits::submit_job!(&ConversationJudgeJob);
