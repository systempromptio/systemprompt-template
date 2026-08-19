//! Prerenders the skills page during `publish_pipeline`.
//!
//! Categories render in a curated order rather than alphabetically; anything
//! not named in that order sorts last.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::extension::prelude::*;

use super::config::{SkillEntry, SkillsPageConfig};

#[derive(Debug)]
pub struct SkillsPagePrerenderer {
    config: Arc<SkillsPageConfig>,
}

impl SkillsPagePrerenderer {
    #[must_use]
    pub const fn new(config: Arc<SkillsPageConfig>) -> Self {
        Self { config }
    }
}

const CATEGORY_ORDER: [&str; 5] = [
    "Salesforce",
    "Consultancy Workflows",
    "Brand & Workspace",
    "Governance & Analytics",
    "Platform & Operations",
];

#[doc(hidden)]
pub fn category_rank(name: &str) -> usize {
    CATEGORY_ORDER
        .iter()
        .position(|c| *c == name)
        .unwrap_or(CATEGORY_ORDER.len())
}

// JSON: template render data — the grouped categories are handed straight to
// the Handlebars context, which takes data rather than a type.
#[doc(hidden)]
pub fn group_by_category(skills: &[SkillEntry]) -> Vec<serde_json::Value> {
    let mut grouped: BTreeMap<String, Vec<&SkillEntry>> = BTreeMap::new();
    for skill in skills {
        let category = skill
            .display_category
            .clone()
            .or_else(|| skill.category.clone())
            .unwrap_or_else(|| "General".to_owned());
        grouped.entry(category).or_default().push(skill);
    }

    let mut categories: Vec<(String, Vec<&SkillEntry>)> = grouped.into_iter().collect();
    categories.sort_by(|a, b| {
        category_rank(&a.0)
            .cmp(&category_rank(&b.0))
            .then_with(|| a.0.cmp(&b.0))
    });

    categories
        .into_iter()
        .map(|(category, items)| {
            serde_json::json!({
                "name": category,
                "skills": items,
            })
        })
        .collect()
}

#[async_trait]
impl PagePrerenderer for SkillsPagePrerenderer {
    fn page_type(&self) -> &'static str {
        "skills-page"
    }

    fn priority(&self) -> u32 {
        50
    }

    async fn prepare(
        &self,
        ctx: &PagePrepareContext<'_>,
    ) -> Result<Option<PageRenderSpec>, systemprompt::traits::ProviderError> {
        let categories = group_by_category(&self.config.skills);

        let base_data = serde_json::json!({
            "site": ctx.web_config,
            "skills": {
                "items": self.config.skills,
                "categories": categories,
                "count": self.config.skills.len(),
            },
        });

        Ok(Some(PageRenderSpec::new(
            "skills",
            base_data,
            PathBuf::from("skills/index.html"),
        )))
    }
}
