//! Shared builders for the gateway evidence modules: a provider block with a
//! chosen governance posture and a catalog resolved from YAML the way the
//! production loader resolves `services/ai`.
//!
//! `enabled: false` on every gateway keeps `ResolvedCatalog::build` off the
//! deployment-pricing tier, which is billing evidence, not the
//! residency/exposure evidence these modules are about; route governance is
//! checked regardless of `enabled`.

use systemprompt::manifest::services::catalog::{
    CatalogContext, CatalogDocument, CatalogError, ResolvedCatalog,
};

pub(crate) struct Provider<'a> {
    pub name: &'a str,
    pub wire: &'a str,
    pub european: bool,
    pub no_retain: bool,
}

pub(crate) const fn provider<'a>(name: &'a str, wire: &'a str) -> Provider<'a> {
    Provider {
        name,
        wire,
        european: false,
        no_retain: false,
    }
}

pub(crate) fn catalog(
    providers: &[Provider<'_>],
    body: &str,
) -> Result<ResolvedCatalog, CatalogError> {
    let mut yaml = String::from("credentials:\n");
    for p in providers {
        yaml.push_str(&format!(
            "  {name}: {{ kind: platform, secret: {name}_key }}\n",
            name = p.name
        ));
    }
    yaml.push_str("providers:\n");
    for p in providers {
        yaml.push_str(&format!(
            "  {name}:\n    credential: {name}\n    governance: {{ european: {eu}, no_retain: {nr} }}\n    endpoints:\n      api: {{ wire: {wire}, url: \"https://{name}.example.com/v1\" }}\n",
            name = p.name,
            eu = p.european,
            nr = p.no_retain,
            wire = p.wire,
        ));
    }
    yaml.push_str(body);
    CatalogDocument::from_yaml_str("gateway-evidence", &yaml)
        .expect("the evidence catalog parses")
        .resolve(&CatalogContext::default())
}
