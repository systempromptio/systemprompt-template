//! Every public-site partial is registered with the template engine under its
//! `component_id` and injected into pages under its `variable_name`. A
//! collision in either is silent at build time and shows up as a page rendering
//! the wrong fragment, so uniqueness is asserted across the whole set. The
//! partials also carry an embedded template — they exist to compile markup into
//! the binary, so a `None` there means the fragment would never render.

use std::sync::Arc;
use systemprompt::template_provider::{ComponentRenderer, PartialSource};
use systemprompt_web_site::partials::{
    AgenticMeshAnimationPartialRenderer, ArchitectureDiagramPartialRenderer,
    CliRemoteAnimationPartialRenderer, FooterPartialRenderer, HeadAssetsPartialRenderer,
    HeaderPartialRenderer, MemoryLoopAnimationPartialRenderer, RustMeshAnimationPartialRenderer,
    ScriptsPartialRenderer,
};

fn all_renderers() -> Vec<Arc<dyn ComponentRenderer>> {
    vec![
        Arc::new(HeadAssetsPartialRenderer),
        Arc::new(HeaderPartialRenderer),
        Arc::new(FooterPartialRenderer),
        Arc::new(ScriptsPartialRenderer),
        Arc::new(CliRemoteAnimationPartialRenderer),
        Arc::new(RustMeshAnimationPartialRenderer),
        Arc::new(MemoryLoopAnimationPartialRenderer),
        Arc::new(AgenticMeshAnimationPartialRenderer),
        Arc::new(ArchitectureDiagramPartialRenderer),
    ]
}

#[test]
fn component_ids_are_unique_and_namespaced_to_the_web_extension() {
    let mut ids: Vec<&str> = all_renderers().iter().map(|r| r.component_id()).collect();
    for id in &ids {
        assert!(
            id.starts_with("web:"),
            "{id} must be namespaced so it cannot collide with another extension"
        );
    }
    ids.sort_unstable();
    let count = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), count, "two partials share a component id");
}

#[test]
fn variable_names_are_unique_and_non_empty() {
    let mut names: Vec<&str> = all_renderers().iter().map(|r| r.variable_name()).collect();
    for name in &names {
        assert!(!name.is_empty(), "a partial has no template variable name");
    }
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(
        names.len(),
        count,
        "two partials write to the same template variable"
    );
}

#[test]
fn every_partial_embeds_a_non_empty_template() {
    for renderer in all_renderers() {
        let template = renderer
            .partial_template()
            .unwrap_or_else(|| panic!("{} embeds a template", renderer.component_id()));
        assert!(!template.name.is_empty());
        match template.source {
            PartialSource::Embedded(markup) => assert!(
                !markup.trim().is_empty(),
                "{} embedded an empty template",
                renderer.component_id()
            ),
            PartialSource::File(path) => panic!(
                "{} must compile its markup into the binary, not read {}",
                renderer.component_id(),
                path.display()
            ),
        }
        assert!(
            renderer.priority() > 0,
            "{} must declare a render priority",
            renderer.component_id()
        );
        assert!(
            renderer.applies_to().is_empty(),
            "{} is a site-wide partial and must not restrict itself to page types",
            renderer.component_id()
        );
    }
}
