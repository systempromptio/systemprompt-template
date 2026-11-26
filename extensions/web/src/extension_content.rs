//! The content member's contributions from the web extension: page data
//! providers, prerenderers, template extenders and the site partials.

use std::sync::Arc;

use systemprompt::content::contract::ContentSlot;
use systemprompt::extension::prelude::*;
use systemprompt_web_shared::config_loader::branding_config;
use systemprompt_web_site::config_loader;
use systemprompt_web_site::partials::{
    FooterPartialRenderer, HeadAssetsPartialRenderer, HeaderPartialRenderer, ScriptsPartialRenderer,
};
use systemprompt_web_site::partials_animations::{
    AgenticMeshAnimationPartialRenderer, ArchitectureDiagramPartialRenderer,
    CliRemoteAnimationPartialRenderer, MemoryLoopAnimationPartialRenderer,
    RustMeshAnimationPartialRenderer,
};

use crate::homepage::{HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::NavigationPageDataProvider;
use crate::{
    DocsContentDataProvider, DocsPageDataProvider, OrgUrlExtender, ReleaseVersionExtender,
    SkillsPagePrerenderer, WebSettings,
};

fn services(r: &Resolved<'_>) -> Result<Arc<WebSettings>, FactoryError> {
    Ok(r.get::<Arc<WebSettings>>()?)
}

// Why: a content factory cannot decline (`ContentSlotBuilder` takes a
// factory that returns the provider or fails), so a page whose config file is
// missing refuses the slot rather than rendering from an invented default.
#[derive(Debug, thiserror::Error)]
#[error("{0} is not configured in the services tree")]
struct MissingSiteConfig(&'static str);

fn required<T>(config: Option<Arc<T>>, what: &'static str) -> Result<Arc<T>, FactoryError> {
    config.ok_or_else(|| FactoryError::Failed(Box::new(MissingSiteConfig(what))))
}

pub(crate) fn contribute_content(c: &mut Contributions<'_>) {
    let content = c.slot::<ContentSlot>();
    content
        .page_data_provider(
            |r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                let settings = services(r)?;
                let nav = required(
                    config_loader::navigation_config(&settings.services_path),
                    "web/navigation.yaml",
                )?;
                let branding = branding_config(&settings.services_path);
                Ok(Arc::new(
                    NavigationPageDataProvider::new(nav).with_branding(branding),
                ))
            },
        )
        .page_data_provider(
            |r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                let settings = services(r)?;
                let homepage =
                    config_loader::homepage_config(&settings.services_path).unwrap_or_default();
                Ok(Arc::new(HomepagePageDataProvider::new(homepage)))
            },
        )
        .page_prerenderer(
            |r: &Resolved<'_>| -> Result<Arc<dyn PagePrerenderer>, FactoryError> {
                let settings = services(r)?;
                let homepage =
                    config_loader::homepage_config(&settings.services_path).unwrap_or_default();
                Ok(Arc::new(HomepagePrerenderer::new(homepage)))
            },
        )
        .page_prerenderer(
            |r: &Resolved<'_>| -> Result<Arc<dyn PagePrerenderer>, FactoryError> {
                let settings = services(r)?;
                let skills = required(
                    config_loader::skills_page_config(&settings.services_path),
                    "web/skills page config",
                )?;
                Ok(Arc::new(SkillsPagePrerenderer::new(skills)))
            },
        )
        .page_data_provider(
            |_r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                Ok(Arc::new(DocsPageDataProvider::new()))
            },
        )
        .content_data_provider(
            |_r: &Resolved<'_>| -> Result<Arc<dyn ContentDataProvider>, FactoryError> {
                Ok(Arc::new(DocsContentDataProvider::new()))
            },
        )
        .template_data_extender(
            |r: &Resolved<'_>| -> Result<Arc<dyn TemplateDataExtender>, FactoryError> {
                let settings = services(r)?;
                Ok(Arc::new(OrgUrlExtender::new(
                    settings.api_external_url.clone(),
                )))
            },
        )
        .template_data_extender(
            |_r: &Resolved<'_>| -> Result<Arc<dyn TemplateDataExtender>, FactoryError> {
                Ok(Arc::new(ReleaseVersionExtender::new()))
            },
        );
    contribute_partials(content);
}

macro_rules! component {
    ($content:expr, $renderer:expr) => {
        $content.component_renderer(
            |_r: &Resolved<'_>| -> Result<Arc<dyn ComponentRenderer>, FactoryError> {
                Ok(Arc::new($renderer))
            },
        )
    };
}

fn contribute_partials(content: &mut systemprompt::content::contract::ContentSlotBuilder) {
    component!(content, HeadAssetsPartialRenderer);
    component!(content, HeaderPartialRenderer);
    component!(content, FooterPartialRenderer);
    component!(content, ScriptsPartialRenderer);
    component!(content, CliRemoteAnimationPartialRenderer);
    component!(content, RustMeshAnimationPartialRenderer);
    component!(content, MemoryLoopAnimationPartialRenderer);
    component!(content, AgenticMeshAnimationPartialRenderer);
    component!(content, ArchitectureDiagramPartialRenderer);
}
