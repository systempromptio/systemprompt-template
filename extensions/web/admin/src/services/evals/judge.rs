//! The judge call itself.
//!
//! One gateway `/v1/messages` call per item (see [`super::gateway_client`]),
//! parsed into the typed verdict in [`super::rubric`]. Because the call goes
//! through our own gateway it lands in `ai_requests` like any other client's
//! traffic, which is how the per-run judge cost is a recorded number rather
//! than an estimate.

use sqlx::PgPool;
use systemprompt::identifiers::{GatewayConversationId, UserId};

use super::gateway_client::{self, CallParams, GatewayCredential};
use super::rubric::{
    JUDGE_SYSTEM_PROMPT, JudgeVerdict, PAIRWISE_SYSTEM_PROMPT, PairwiseVerdict, judge_user_prompt,
    pairwise_user_prompt,
};

const JUDGE_MAX_TOKENS: u32 = 2048;

// Why: pairs the grading model with the credential its calls travel under.
#[derive(Debug, Clone)]
pub(crate) struct JudgeConfig {
    pub provider: String,
    pub model: String,
    pub actor_user_id: UserId,
    pub run_id: String,
    pub credential: GatewayCredential,
}

// Why: the cost travels with the verdict so a run can total its own spend.
#[derive(Debug, Clone)]
pub(crate) struct JudgedItem {
    pub verdict: JudgeVerdict,
    pub cost_microdollars: i64,
}

// Why: one prompt/answer pair, one verdict.
pub(crate) async fn judge_answer(
    pool: &PgPool,
    config: &JudgeConfig,
    prompt: &str,
    answer: &str,
) -> Option<JudgedItem> {
    let raw = call_judge(
        pool,
        config,
        JUDGE_SYSTEM_PROMPT,
        &judge_user_prompt(prompt, answer),
    )
    .await?;

    let verdict = parse_reply::<JudgeVerdict>(&raw.text, "judge", &config.run_id)?.normalised();
    let cost = lookup_cost(pool, &raw.conversation_id).await;

    Some(JudgedItem {
        verdict,
        cost_microdollars: cost,
    })
}

// Why: the cost travels with the decision so a run can total its own spend.
#[derive(Debug, Clone)]
pub(crate) struct JudgedPair {
    pub verdict: PairwiseVerdict,
    pub cost_microdollars: i64,
}

#[derive(Debug)]
pub(crate) struct PairParams<'a> {
    pub pool: &'a PgPool,
    pub config: &'a JudgeConfig,
    pub prompt: &'a str,
    pub answer_a: &'a str,
    pub answer_b: &'a str,
}

// Why: callers run this twice with the answers swapped; see `super::pairwise`.
pub(crate) async fn judge_pair(params: PairParams<'_>) -> Option<JudgedPair> {
    let raw = call_judge(
        params.pool,
        params.config,
        PAIRWISE_SYSTEM_PROMPT,
        &pairwise_user_prompt(params.prompt, params.answer_a, params.answer_b),
    )
    .await?;

    let verdict = parse_reply::<PairwiseVerdict>(&raw.text, "pairwise", &params.config.run_id)?;
    let cost = lookup_cost(params.pool, &raw.conversation_id).await;

    Some(JudgedPair {
        verdict,
        cost_microdollars: cost,
    })
}

async fn call_judge(
    pool: &PgPool,
    config: &JudgeConfig,
    system: &str,
    user: &str,
) -> Option<gateway_client::GatewayAnswer> {
    let conversation_id = gateway_client::new_conversation_id();
    // Why: recorded before the call, so a call that fails mid-flight is still
    // excluded from later candidate pools.
    record_call(pool, &conversation_id, &config.run_id).await;

    gateway_client::call_messages(CallParams {
        credential: &config.credential,
        model: &config.model,
        system: Some(system),
        user,
        max_tokens: JUDGE_MAX_TOKENS,
        conversation_id: &conversation_id,
    })
    .await
}

pub(super) async fn record_call(
    pool: &PgPool,
    conversation_id: &GatewayConversationId,
    run_id: &str,
) {
    if let Err(e) = crate::repositories::evals::sampling::insert_judge_call(
        pool,
        conversation_id.as_str(),
        run_id,
    )
    .await
    {
        tracing::warn!(error = %e, "could not record an eval call for later exclusion");
    }
}

fn parse_reply<T: serde::de::DeserializeOwned>(text: &str, what: &str, run_id: &str) -> Option<T> {
    let json = gateway_client::extract_json_object(text).unwrap_or(text);
    serde_json::from_str::<T>(json)
        .inspect_err(|e| {
            tracing::warn!(
                error = %e,
                kind = what,
                run_id,
                reply = %text.chars().take(300).collect::<String>(),
                "eval judge returned an unparseable verdict"
            );
        })
        .ok()
}

async fn lookup_cost(pool: &PgPool, conversation_id: &GatewayConversationId) -> i64 {
    crate::repositories::evals::sampling::find_conversation_cost(pool, conversation_id.as_str())
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(error = %e, "could not read judge request cost");
            None
        })
        .unwrap_or(0)
}
