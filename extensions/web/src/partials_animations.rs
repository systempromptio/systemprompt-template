#![allow(clippy::unnecessary_literal_bound)]

use anyhow::Result;
use async_trait::async_trait;
use systemprompt::template_provider::{
    ComponentContext, ComponentRenderer, PartialTemplate, RenderedComponent,
};

use super::partials::PRIORITY_MID;

pub struct CliRemoteAnimationPartialRenderer;

impl CliRemoteAnimationPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/animation-cli-remote.html");
}

#[async_trait]
impl ComponentRenderer for CliRemoteAnimationPartialRenderer {
    fn component_id(&self) -> &str {
        "web:cli-remote-animation"
    }

    fn variable_name(&self) -> &str {
        "ANIMATION_CLI_REMOTE"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded(
            "animation-cli-remote",
            Self::TEMPLATE,
        ))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_MID
    }
}

pub struct RustMeshAnimationPartialRenderer;

impl RustMeshAnimationPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/animation-rust-mesh.html");
}

#[async_trait]
impl ComponentRenderer for RustMeshAnimationPartialRenderer {
    fn component_id(&self) -> &str {
        "web:rust-mesh-animation"
    }

    fn variable_name(&self) -> &str {
        "RUST_MESH_ANIMATION"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded(
            "rust-mesh-animation",
            Self::TEMPLATE,
        ))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_MID
    }
}

pub struct MemoryLoopAnimationPartialRenderer;

impl MemoryLoopAnimationPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/animation-memory-loop.html");
}

#[async_trait]
impl ComponentRenderer for MemoryLoopAnimationPartialRenderer {
    fn component_id(&self) -> &str {
        "web:memory-loop-animation"
    }

    fn variable_name(&self) -> &str {
        "ANIMATION_MEMORY_LOOP"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded(
            "animation-memory-loop",
            Self::TEMPLATE,
        ))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_MID
    }
}

pub struct AgenticMeshAnimationPartialRenderer;

impl AgenticMeshAnimationPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/animation-agentic-mesh.html");
}

#[async_trait]
impl ComponentRenderer for AgenticMeshAnimationPartialRenderer {
    fn component_id(&self) -> &str {
        "web:agentic-mesh-animation"
    }

    fn variable_name(&self) -> &str {
        "ANIMATION_AGENTIC_MESH"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded(
            "animation-agentic-mesh",
            Self::TEMPLATE,
        ))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_MID
    }
}

pub struct ArchitectureDiagramPartialRenderer;

impl ArchitectureDiagramPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/architecture-diagram.html");
}

#[async_trait]
impl ComponentRenderer for ArchitectureDiagramPartialRenderer {
    fn component_id(&self) -> &str {
        "web:architecture-diagram"
    }

    fn variable_name(&self) -> &str {
        "ARCHITECTURE_DIAGRAM"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded(
            "architecture-diagram",
            Self::TEMPLATE,
        ))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_MID
    }
}
