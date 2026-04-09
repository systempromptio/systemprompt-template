#![allow(clippy::unnecessary_literal_bound)]

use async_trait::async_trait;
use systemprompt::template_provider::{
    ComponentContext, ComponentRenderer, PartialTemplate, RenderedComponent,
};

pub(crate) const PRIORITY_CRITICAL: u32 = 5;
pub(crate) const PRIORITY_HIGH: u32 = 10;
pub(crate) const PRIORITY_MID: u32 = 50;
pub(crate) const PRIORITY_LOW: u32 = 90;
pub(crate) const PRIORITY_LAST: u32 = 95;

pub use super::partials_animations::{
    AgenticMeshAnimationPartialRenderer, ArchitectureDiagramPartialRenderer,
    CliRemoteAnimationPartialRenderer, MemoryLoopAnimationPartialRenderer,
    RustMeshAnimationPartialRenderer,
};

#[derive(Debug, Clone, Copy)]
pub struct HeadAssetsPartialRenderer;

impl HeadAssetsPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../../services/web/templates/partials/head-assets.html");
}

#[async_trait]
impl ComponentRenderer for HeadAssetsPartialRenderer {
    fn component_id(&self) -> &'static str {
        "web:head-assets-partial"
    }

    fn variable_name(&self) -> &'static str {
        "HEAD_ASSETS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("head-assets", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> anyhow::Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_CRITICAL
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeaderPartialRenderer;

impl HeaderPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../../services/web/templates/partials/header.html");
}

#[async_trait]
impl ComponentRenderer for HeaderPartialRenderer {
    fn component_id(&self) -> &'static str {
        "web:header-partial"
    }

    fn variable_name(&self) -> &'static str {
        "HEADER"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("header", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> anyhow::Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_HIGH
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FooterPartialRenderer;

impl FooterPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../../services/web/templates/partials/footer.html");
}

#[async_trait]
impl ComponentRenderer for FooterPartialRenderer {
    fn component_id(&self) -> &'static str {
        "web:footer-partial"
    }

    fn variable_name(&self) -> &'static str {
        "FOOTER"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("footer", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> anyhow::Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_LOW
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScriptsPartialRenderer;

impl ScriptsPartialRenderer {
    const TEMPLATE: &'static str =
        include_str!("../../../../services/web/templates/partials/scripts.html");
}

#[async_trait]
impl ComponentRenderer for ScriptsPartialRenderer {
    fn component_id(&self) -> &'static str {
        "web:scripts-partial"
    }

    fn variable_name(&self) -> &'static str {
        "SCRIPTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate::embedded("scripts", Self::TEMPLATE))
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> anyhow::Result<RenderedComponent> {
        Ok(RenderedComponent::new(self.variable_name(), ""))
    }

    fn priority(&self) -> u32 {
        PRIORITY_LAST
    }
}
