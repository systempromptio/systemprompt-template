//! Generating an AI report in the moment. `POST /admin/analysis/reports`
//! writes the row with its digest and a lease, then hands it to
//! [`generate_report`] on a background task; the report page polls the row
//! until it is `generated` or `failed`. There is no queue and no scheduler:
//! a report exists because someone asked for it, and it is written by one
//! structured-output call to the judge's model, audited under a job actor
//! (`analysis_report`) so the call is never counted as a conversation.

mod prompt;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::loader::ConfigLoader;
use systemprompt::models::execution::context::RequestContext;

use crate::error::{AdminError, AdminResult};
use crate::repositories::analysis::reports::{
    AnalysisReportRow, ReportCompletion, fail_report, render_digest_text, update_report_completion,
};

pub(crate) use prompt::{findings_schema, parse_findings};

const ACTOR_JOB_NAME: &str = "analysis_report";
const AGENT_NAME: &str = "analysis-reporter";
const MAX_OUTPUT_TOKENS: u32 = 4_096;

// Why: the reporter's own requests need a context of their own, distinct
// from the judge's, so each source of spend can be told apart in the audit.
const CONTEXT_SEED: &[u8] = b"template:analysis_report";

// Why: the provider and model a report is written with.
#[derive(Debug, Clone)]
pub(crate) struct ReportModel {
    pub(crate) provider: String,
    pub(crate) model: String,
}

impl ReportModel {
    // Why: the judge's model, so every AI reading of the record is written
    // by the same model: the `conversation_judge` scheduler entry's
    // parameters, else the default provider's default model.
    pub(crate) fn configured() -> AdminResult<Self> {
        let services = ConfigLoader::load().map_err(AdminError::internal)?;
        let judge = services
            .scheduler
            .as_ref()
            .and_then(|s| s.jobs.iter().find(|j| j.name == "conversation_judge"));
        let provider = judge
            .and_then(|j| j.parameters.get("provider").cloned())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| services.ai.default_provider.clone());
        let model = judge
            .and_then(|j| j.parameters.get("model").cloned())
            .filter(|m| !m.is_empty())
            .or_else(|| {
                services
                    .ai
                    .providers
                    .get(&provider)
                    .map(|p| p.default_model.clone())
                    .filter(|m| !m.is_empty())
            })
            .ok_or_else(|| {
                AdminError::internal(format!("no model is configured for provider {provider}"))
            })?;
        Ok(Self { provider, model })
    }
}

fn request_context(actor: Actor) -> AdminResult<RequestContext> {
    Ok(RequestContext::new(
        SessionId::new(""),
        TraceId::new(uuid::Uuid::new_v4().to_string()),
        ContextId::from_uuid(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, CONTEXT_SEED)),
        AgentName::try_new(AGENT_NAME).map_err(AdminError::internal)?,
    )
    .with_actor(actor))
}

fn digest_text(row: &AnalysisReportRow) -> String {
    let digest = render_digest_text(&row.inputs.digest);
    let scope = row.inputs.filter_label.as_deref().map_or_else(
        || {
            row.scope_id
                .as_deref()
                .map(|id| format!(" {id}"))
                .unwrap_or_default()
        },
        |label| format!(" — {label}"),
    );
    format!("SCOPE {}{scope}\n{digest}", row.scope_kind)
}

// Why: who asked and the lease the row was inserted with — the two facts
// the completion must present together.
struct Grant<'a> {
    requested_by: &'a UserId,
    lease: &'a str,
}

async fn call_model(
    pool: &PgPool,
    ai: &AiService,
    grant: Grant<'_>,
    row: &AnalysisReportRow,
    model: &ReportModel,
) -> AdminResult<()> {
    let request = AiRequest::builder(
        vec![
            AiMessage::system(prompt::SYSTEM_PROMPT),
            AiMessage::user(digest_text(row)),
        ],
        model.provider.as_str(),
        model.model.as_str(),
        MAX_OUTPUT_TOKENS,
        request_context(Actor::job(grant.requested_by.clone(), ACTOR_JOB_NAME))?,
    )
    .with_structured_output(StructuredOutputOptions::with_schema(findings_schema()))
    .build();
    let response = ai.generate(&request).await.map_err(AdminError::internal)?;
    let findings = parse_findings(&response.content).map_err(AdminError::internal)?;
    let request_id = response.request_id.to_string();
    let written = update_report_completion(
        pool,
        ReportCompletion {
            id: &row.id,
            lease_token: grant.lease,
            findings: &findings,
            provider: &model.provider,
            model: &model.model,
            ai_request_id: Some(request_id.as_str()),
            input_tokens: response.input_tokens.and_then(|t| i32::try_from(t).ok()),
            output_tokens: response.output_tokens.and_then(|t| i32::try_from(t).ok()),
        },
    )
    .await?;
    if !written {
        tracing::warn!(report_id = %row.id, "analysis report lease was lost before completion");
    }
    Ok(())
}

// Why: one report, start to finish: the model call, then the row written
// as generated or as failed with the reason the page shows. Runs on a
// spawned task, so every outcome is recorded rather than returned.
pub(crate) async fn generate_report(
    pool: Arc<PgPool>,
    ai: Option<Arc<AiService>>,
    requested_by: UserId,
    row: AnalysisReportRow,
    lease: String,
) {
    let outcome = match ai {
        None => Err(AdminError::internal(
            "no AI provider is configured on this instance, so a report cannot be written",
        )),
        Some(ai) => match ReportModel::configured() {
            Err(error) => Err(error),
            Ok(model) => {
                let grant = Grant {
                    requested_by: &requested_by,
                    lease: &lease,
                };
                call_model(&pool, &ai, grant, &row, &model).await
            },
        },
    };
    if let Err(error) = outcome {
        tracing::warn!(report_id = %row.id, %error, "analysis report failed");
        if let Err(write) = fail_report(&pool, &row.id, &lease, &error.to_string()).await {
            tracing::error!(report_id = %row.id, %write, "analysis report failure could not be recorded");
        }
    }
}
