//! Scheduler parameters for one conversation-judge tick.

use systemprompt::identifiers::ContextId;
use systemprompt::traits::JobContext;

use crate::JobError;

/// Tunables accepted from the scheduler entry or a manual job invocation.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct JudgeParams {
    pub provider: String,
    pub model: String,
    pub batch_size: u32,
    pub daily_cost_cap_microdollars: i64,
    pub quiet_minutes: i32,
    pub lookback_days: i32,
    pub transcript_token_budget: usize,
    pub max_output_tokens: u32,
    pub context_id: Option<ContextId>,
}

impl Default for JudgeParams {
    fn default() -> Self {
        Self {
            provider: "gemini".to_owned(),
            model: "gemini-3.8-flash".to_owned(),
            batch_size: 20,
            daily_cost_cap_microdollars: 2_000_000,
            quiet_minutes: 30,
            lookback_days: 30,
            transcript_token_budget: 24_000,
            max_output_tokens: 2_048,
            context_id: None,
        }
    }
}

impl JudgeParams {
    #[doc(hidden)]
    pub fn from_context(ctx: &JobContext) -> Result<Self, JobError> {
        let defaults = Self::default();
        let parse = |key: &str| -> Result<Option<String>, JobError> {
            Ok(ctx.get_parameter_parsed::<String>(key)?)
        };
        let num = |key: &str| -> Result<Option<i64>, JobError> {
            Ok(ctx.get_parameter_parsed::<i64>(key)?)
        };
        let batch = num("batch_size")?.unwrap_or_else(|| i64::from(defaults.batch_size));
        let max_out =
            num("max_output_tokens")?.unwrap_or_else(|| i64::from(defaults.max_output_tokens));
        let default_budget = i64::try_from(defaults.transcript_token_budget).unwrap_or(i64::MAX);
        let budget = num("transcript_token_budget")?.unwrap_or(default_budget);
        Ok(Self {
            provider: parse("provider")?.unwrap_or(defaults.provider),
            model: parse("model")?.unwrap_or(defaults.model),
            batch_size: u32::try_from(batch.clamp(1, 100)).unwrap_or(defaults.batch_size),
            daily_cost_cap_microdollars: num("daily_cost_cap_microdollars")?
                .unwrap_or(defaults.daily_cost_cap_microdollars),
            quiet_minutes: num("quiet_minutes")?.map_or(defaults.quiet_minutes, |v| {
                i32::try_from(v.clamp(0, 525_600)).unwrap_or(0)
            }),
            lookback_days: num("lookback_days")?.map_or(defaults.lookback_days, |v| {
                i32::try_from(v.clamp(1, 3_660)).unwrap_or(30)
            }),
            transcript_token_budget: usize::try_from(budget.clamp(1_000, 400_000))
                .unwrap_or(defaults.transcript_token_budget),
            max_output_tokens: u32::try_from(max_out.clamp(256, 65_536))
                .unwrap_or(defaults.max_output_tokens),
            context_id: parse("context_id")?.map(ContextId::try_new).transpose()?,
        })
    }
}
