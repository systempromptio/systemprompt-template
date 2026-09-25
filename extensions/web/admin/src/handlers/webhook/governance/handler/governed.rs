//! Projection of a raw hook payload onto the policy engine's governed types.

use systemprompt::identifiers::McpToolName;
use systemprompt_security::policy::{GovernedInput, GovernedTarget, McpToolInput};

use crate::types::webhook::HookEventPayload;

pub(super) fn governed_target(payload: &HookEventPayload) -> GovernedTarget {
    payload.tool_name().map_or_else(
        || {
            if payload.prompt().is_some() {
                GovernedTarget::Prompt
            } else {
                GovernedTarget::Unknown
            }
        },
        |name| {
            McpToolName::try_new(name).map_or(GovernedTarget::Unknown, |tool| {
                GovernedTarget::Tool { tool }
            })
        },
    )
}

pub(super) fn governed_input(payload: &HookEventPayload) -> GovernedInput {
    payload.prompt().map_or_else(
        || {
            GovernedInput::tool_arguments(McpToolInput::new(
                payload.tool_input().cloned().unwrap_or_default(),
            ))
        },
        |text| GovernedInput::prompt_text(text.to_owned()),
    )
}
