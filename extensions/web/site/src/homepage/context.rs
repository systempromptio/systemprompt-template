use serde::Serialize;

use super::config::HomepageConfig;

/// Template context for the homepage (`homepage.html`), shared by the runtime
/// [`super::provider::HomepagePageDataProvider`] and the build-time
/// [`super::prerenderer::HomepagePrerenderer`].
///
/// The template reads the homepage configuration under `site.homepage.*`.
#[derive(Debug, Serialize)]
pub(super) struct HomepageContext<'a> {
    site: HomepageSite<'a>,
}

#[derive(Debug, Serialize)]
struct HomepageSite<'a> {
    homepage: &'a HomepageConfig,
}

impl<'a> HomepageContext<'a> {
    pub(super) const fn new(homepage: &'a HomepageConfig) -> Self {
        Self {
            site: HomepageSite { homepage },
        }
    }
}
