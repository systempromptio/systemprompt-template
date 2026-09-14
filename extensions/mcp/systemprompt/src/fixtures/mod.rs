//! Evaluation-only fixture MCP adapter with a closed operation set.

use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{McpOutputSchema, McpToolHandler};
use systemprompt::models::execution::context::RequestContext;

mod approvals;
mod error;
mod records;

use error::FixtureError;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FixtureOperation {
    AtlassianSearch,
    AtlassianRead,
    PlatformUsageRead,
    PlatformTestRecordRead,
    PlatformTestRecordWrite,
    PlatformTestRecordRestore,
}

impl FixtureOperation {
    // Why: the wire name is the serde name, so the evidence label a verifier
    // reads back matches the operation a client asked for.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AtlassianSearch => "atlassian_search",
            Self::AtlassianRead => "atlassian_read",
            Self::PlatformUsageRead => "platform_usage_read",
            Self::PlatformTestRecordRead => "platform_test_record_read",
            Self::PlatformTestRecordWrite => "platform_test_record_write",
            Self::PlatformTestRecordRestore => "platform_test_record_restore",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
// JSON: Fixture values are protocol-boundary documents with case-specific
// shapes.
pub struct FixtureInput {
    pub operation: FixtureOperation,
    pub fixture_key: String,
    pub expected_digest: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, JsonSchema)]
// JSON: Fixture payloads preserve the exact case-specific evidence document.
pub struct FixtureOutput {
    pub fixture: bool,
    pub evidence_label: String,
    pub operation: String,
    pub digest: String,
    pub payload: serde_json::Value,
    pub readback_verified: bool,
}

impl McpOutputSchema for FixtureOutput {
    fn artifact_type() -> &'static str {
        "list"
    }
    fn artifact_title(&self) -> Option<String> {
        Some("Evaluation fixture evidence".to_owned())
    }
    fn text_body(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

#[derive(Debug)]
pub struct FixtureHandler<'a> {
    pub pool: &'a DbPool,
}

impl McpToolHandler for FixtureHandler<'_> {
    type Input = FixtureInput;
    type Output = FixtureOutput;
    fn tool_name(&self) -> &'static str {
        "evaluation_fixture"
    }
    fn description(&self) -> &'static str {
        "Execute one explicitly allowed deterministic fixture operation; results are always labelled fixture evidence."
    }

    async fn handle(
        &self,
        input: Self::Input,
        context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let evidence = self.run(&input, context).await?;
        Ok((
            FixtureOutput {
                fixture: true,
                evidence_label: evidence.label,
                operation: input.operation.as_str().to_owned(),
                digest: evidence.digest,
                payload: evidence.payload,
                readback_verified: evidence.readback_verified,
            },
            evidence.message.to_owned(),
        ))
    }
}

impl FixtureHandler<'_> {
    async fn run(
        &self,
        input: &FixtureInput,
        context: &RequestContext,
    ) -> Result<records::Evidence, FixtureError> {
        validate_key(&input.fixture_key)?;
        let owner = context.user_id();
        let read_pool = self.pool.pool().ok_or(FixtureError::Unavailable(
            "PostgreSQL evaluation database unavailable",
        ))?;
        match input.operation {
            FixtureOperation::AtlassianSearch
            | FixtureOperation::AtlassianRead
            | FixtureOperation::PlatformUsageRead => {
                records::read_payload(read_pool.as_ref(), owner, input).await
            },
            FixtureOperation::PlatformTestRecordRead => {
                records::read_record(read_pool.as_ref(), owner, input).await
            },
            FixtureOperation::PlatformTestRecordWrite => {
                let value = input
                    .value
                    .clone()
                    .ok_or(FixtureError::Rejected("Write requires a value"))?;
                records::mutate_record(self.pool, context, input, records::Mutation::Write(value))
                    .await
            },
            FixtureOperation::PlatformTestRecordRestore => {
                records::mutate_record(self.pool, context, input, records::Mutation::Restore).await
            },
        }
    }
}

fn validate_key(value: &str) -> Result<(), FixtureError> {
    if value.is_empty()
        || value.len() > 200
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(FixtureError::Rejected("Invalid fixture key"));
    }
    Ok(())
}
// JSON: Canonical JCS hashing accepts every valid fixture document shape.
fn digest(value: &serde_json::Value) -> Result<String, FixtureError> {
    use sha2::{Digest, Sha256};
    let bytes = serde_jcs::to_vec(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
