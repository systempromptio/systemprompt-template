#![allow(clippy::unnecessary_literal_bound)]

use anyhow::Result;
use async_trait::async_trait;
use systemprompt::template_provider::{
    ComponentContext, ComponentRenderer, PartialTemplate, RenderedComponent,
};

const PRIORITY_CRITICAL: u32 = 5;
const PRIORITY_HIGH: u32 = 10;
const PRIORITY_MID: u32 = 50;
const PRIORITY_LOW: u32 = 90;
const PRIORITY_LAST: u32 = 95;

pub struct HeadAssetsPartialRenderer;

impl HeadAssetsPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/head-assets.html");
}

#[async_trait]
impl ComponentRenderer for HeadAssetsPartialRenderer {
    fn component_id(&self) -> &str {
        "web:head-assets-partial"
    }

    fn variable_name(&self) -> &str {
        "HEAD_ASSETS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("head-assets", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_CRITICAL
    }
}

pub struct HeaderPartialRenderer;

impl HeaderPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/header.html");
}

#[async_trait]
impl ComponentRenderer for HeaderPartialRenderer {
    fn component_id(&self) -> &str {
        "web:header-partial"
    }

    fn variable_name(&self) -> &str {
        "HEADER"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("header", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_HIGH
    }
}

pub struct FooterPartialRenderer;

impl FooterPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/footer.html");
}

#[async_trait]
impl ComponentRenderer for FooterPartialRenderer {
    fn component_id(&self) -> &str {
        "web:footer-partial"
    }

    fn variable_name(&self) -> &str {
        "FOOTER"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("footer", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_LOW
    }
}

pub struct ScriptsPartialRenderer;

impl ScriptsPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../services/web/templates/partials/scripts.html");
}

#[async_trait]
impl ComponentRenderer for ScriptsPartialRenderer {
    fn component_id(&self) -> &str {
        "web:scripts-partial"
    }

    fn variable_name(&self) -> &str {
        "SCRIPTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("scripts", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_LAST
    }
}

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
