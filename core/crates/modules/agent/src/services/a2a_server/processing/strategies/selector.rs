//! Strategy Selector
//!
//! Selects the appropriate execution strategy based on agent configuration.

use super::{
    AgenticExecutionStrategy, ExecutionStrategy, StandardExecutionStrategy, ToolExecutionStrategy,
};

#[derive(Debug, Clone, Copy)]
pub struct ExecutionStrategySelector;

impl ExecutionStrategySelector {
    pub fn new() -> Self {
        Self
    }

    pub fn select_strategy(
        &self,
        has_tools: bool,
        force_agentic: bool,
    ) -> Box<dyn ExecutionStrategy> {
        match (has_tools, force_agentic) {
            (_, true) => Box::new(AgenticExecutionStrategy::new()),
            (true, false) => Box::new(ToolExecutionStrategy::new()),
            (false, false) => Box::new(StandardExecutionStrategy::new()),
        }
    }
}

impl Default for ExecutionStrategySelector {
    fn default() -> Self {
        Self::new()
    }
}
