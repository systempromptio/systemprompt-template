//! Strategy Selector
//!
//! Selects the appropriate execution strategy based on agent configuration.
//!
//! Two strategies:
//! - StandardExecutionStrategy: No tools, simple streaming text generation
//! - PlannedAgenticStrategy: Has tools, deterministic plan → execute →
//!   synthesize
//!
//! All tool-based execution uses PlannedAgenticStrategy for predictable,
//! parallel execution.

use super::{ExecutionStrategy, PlannedAgenticStrategy, StandardExecutionStrategy};

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionStrategySelector;

impl ExecutionStrategySelector {
    pub const fn new() -> Self {
        Self
    }

    pub fn select_strategy(&self, has_tools: bool) -> Box<dyn ExecutionStrategy> {
        if has_tools {
            Box::new(PlannedAgenticStrategy::new())
        } else {
            Box::new(StandardExecutionStrategy::new())
        }
    }
}
