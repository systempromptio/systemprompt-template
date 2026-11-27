use std::sync::Arc;

use systemprompt_engine::run::extension::EnabledExtensions;
use systemprompt_engine::run::{
    Endpoint, GovernError, Governor, McpSpec, ModelCall, ModelSpec, ResponseAction, ToolCall,
    Usage, Verdict,
};
use systemprompt_harness_views::app::governor::GovernorSlot;
use systemprompt_identifiers::extension_id;

#[derive(Debug)]
struct Offering;

#[async_trait::async_trait]
impl Governor for Offering {
    fn models(&self) -> Vec<ModelSpec> {
        vec![ModelSpec {
            id: "m".to_owned(),
            name: "M".to_owned(),
            protocol: systemprompt_engine::run::Protocol::Messages,
            context_window: 0,
            max_output: 0,
            supports: systemprompt_engine::llm::Supports::default(),
        }]
    }
    fn mcp_servers(&self) -> Vec<McpSpec> {
        Vec::new()
    }
    async fn authorize(&self, _call: &ModelCall) -> Result<Endpoint, GovernError> {
        Err(GovernError::NotSignedIn)
    }
    async fn response(&self, _call: &ModelCall, _status: u16, _attempt: u32) -> ResponseAction {
        ResponseAction::Continue
    }
    async fn step_finished(&self, _call: &ModelCall, _usage: &Usage) {}
    async fn tool_before(&self, _call: &ToolCall) -> Verdict {
        Verdict::Deny {
            reason: "no".to_owned(),
        }
    }
    fn extensions(&self) -> Option<EnabledExtensions> {
        Some(std::iter::once(extension_id!("drive")).collect())
    }
    fn generation(&self) -> u64 {
        0
    }
}

#[test]
fn installing_switches_delegation_and_bumps_the_generation() {
    let slot = GovernorSlot::default();
    assert_eq!(slot.models(), Vec::<ModelSpec>::new());
    assert_eq!(slot.extensions(), None);
    let before = slot.generation();
    slot.install(Arc::new(Offering));
    assert_eq!(slot.models().len(), 1);
    assert!(
        slot.extensions()
            .is_some_and(|enabled| enabled.contains(&extension_id!("drive")))
    );
    assert_ne!(slot.generation(), before);
}
