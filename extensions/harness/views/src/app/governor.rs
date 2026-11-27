//! The engine's governor seen through one indirection, so the real one can be
//! installed after the engine is built.
//!
//! The engine holds an `Arc<dyn Governor>` for life. The shell gives it a
//! `GovernorSlot`, which delegates every call, the enabled extensions
//! included, to whichever governor is installed: the engine's `NullGovernor`
//! (no models, no extensions) until the composition root installs
//! `systemprompt_harness::governance::Governance` before the engine is built.
//! Installing bumps `Governor::generation`, so the engine re-reads models and
//! MCP servers at once.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use systemprompt_engine::run::extension::EnabledExtensions;
use systemprompt_engine::run::{
    Endpoint, GovernError, Governor, McpSpec, ModelCall, ModelSpec, NullGovernor, ResponseAction,
    ToolCall, Usage, Verdict,
};

const EPOCH_SHIFT: u32 = 48;

/// The governor the engine holds for life, delegating to whichever governor is
/// installed.
pub struct GovernorSlot {
    current: RwLock<Arc<dyn Governor>>,
    epoch: AtomicU64,
}

impl std::fmt::Debug for GovernorSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernorSlot")
            .field("epoch", &self.epoch.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Default for GovernorSlot {
    fn default() -> Self {
        Self::with(Arc::new(NullGovernor))
    }
}

impl GovernorSlot {
    pub(crate) fn with(governor: Arc<dyn Governor>) -> Self {
        Self {
            current: RwLock::new(governor),
            epoch: AtomicU64::new(0),
        }
    }

    pub fn install(&self, governor: Arc<dyn Governor>) {
        *self.current.write() = governor;
        self.epoch.fetch_add(1, Ordering::AcqRel);
        tracing::info!("governor installed");
    }

    pub(crate) fn current(&self) -> Arc<dyn Governor> {
        Arc::clone(&self.current.read())
    }
}

#[async_trait::async_trait]
impl Governor for GovernorSlot {
    fn models(&self) -> Vec<ModelSpec> {
        self.current().models()
    }

    fn mcp_servers(&self) -> Vec<McpSpec> {
        self.current().mcp_servers()
    }

    async fn authorize(&self, call: &ModelCall) -> Result<Endpoint, GovernError> {
        let governor = self.current();
        governor.authorize(call).await
    }

    async fn response(&self, call: &ModelCall, status: u16, attempt: u32) -> ResponseAction {
        let governor = self.current();
        governor.response(call, status, attempt).await
    }

    async fn step_finished(&self, call: &ModelCall, usage: &Usage) {
        let governor = self.current();
        governor.step_finished(call, usage).await;
    }

    async fn tool_before(&self, call: &ToolCall) -> Verdict {
        let governor = self.current();
        governor.tool_before(call).await
    }

    fn extensions(&self) -> Option<EnabledExtensions> {
        self.current().extensions()
    }

    fn generation(&self) -> u64 {
        let epoch = self.epoch.load(Ordering::Acquire);
        (epoch << EPOCH_SHIFT).wrapping_add(self.current().generation())
    }

    fn governed_by(&self) -> Option<String> {
        self.current().governed_by()
    }
}
