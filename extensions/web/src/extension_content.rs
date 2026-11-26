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

type Built<T> = Result<Option<Arc<T>>, FactoryError>;

pub(crate) fn contribute_content(c: &mut Contributions<'_>) {
    let content = c.slot::<ContentSlot>();
    content
        .page_data_provider(|r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            let settings = services(r)?;
            let Some(nav) = config_loader::navigation_config(&settings.services_path) else {
                return Ok(None);
            };
            let branding = branding_config(&settings.services_path);
            Ok(Some(Arc::new(
                NavigationPageDataProvider::new(nav).with_branding(branding),
            )))
        })
        .page_data_provider(|r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            let settings = services(r)?;
            let homepage =
                config_loader::homepage_config(&settings.services_path).unwrap_or_default();
            Ok(Some(Arc::new(HomepagePageDataProvider::new(homepage))))
        })
        .page_prerenderer(|r: &Resolved<'_>| -> Built<dyn PagePrerenderer> {
            let settings = services(r)?;
            let homepage =
                config_loader::homepage_config(&settings.services_path).unwrap_or_default();
            Ok(Some(Arc::new(HomepagePrerenderer::new(homepage))))
        })
        .page_prerenderer(|r: &Resolved<'_>| -> Built<dyn PagePrerenderer> {
            let settings = services(r)?;
            Ok(
                config_loader::skills_page_config(&settings.services_path).map(
                    |skills| -> Arc<dyn PagePrerenderer> {
                        Arc::new(SkillsPagePrerenderer::new(skills))
                    },
                ),
            )
        })
        .page_data_provider(|_r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            Ok(Some(Arc::new(DocsPageDataProvider::new())))
        })
        .content_data_provider(|_r: &Resolved<'_>| -> Built<dyn ContentDataProvider> {
            Ok(Some(Arc::new(DocsContentDataProvider::new())))
        })
        .template_data_extender(|r: &Resolved<'_>| -> Built<dyn TemplateDataExtender> {
            let settings = services(r)?;
            Ok(Some(Arc::new(OrgUrlExtender::new(
                settings.api_external_url.clone(),
            ))))
        })
        .template_data_extender(|_r: &Resolved<'_>| -> Built<dyn TemplateDataExtender> {
            Ok(Some(Arc::new(ReleaseVersionExtender::new())))
        });
    contribute_partials(content);
}

macro_rules! component {
    ($content:expr, $renderer:expr) => {
        $content.component_renderer(|_r: &Resolved<'_>| -> Built<dyn ComponentRenderer> {
            Ok(Some(Arc::new($renderer)))
        })
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