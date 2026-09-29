//! `/admin/analysis/skills` — every skill as the record shows it, in three
//! views under one scope. The scope is the window and, optionally, one
//! marketplace; every tab honours both. *Overview* is the zoomed-out view:
//! one row per marketplace, entitled → installed → active with the window's
//! activity and spend. *Activity* is invocations per day over the window.
//! *Skills* is the table: one row per skill, grouped under the marketplace
//! and plugin that serve it, with a fourteen-day sparkline per row.
//!
//! Invocations are hook events; everything else is read from the
//! `conversation_facts` rollup through the harness session, so a figure here
//! is the same figure the Conversations page shows for the same conversation.
//! Only the active tab's data is read.

mod activity;
mod adoption;
mod context;
mod rows;
mod summary;
mod views;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::inventory_index::{InventoryIndex, get_marketplace_audience};
use crate::repositories::analysis::skills::{
    SkillListFilter, SkillSort, SkillWindow, list_marketplace_adoption, list_skill_facts,
};
use crate::repositories::scope::ScopeRequest;

use context::{SkillsPageInputs, SkillsRead, build_context};
pub(crate) use rows::skill_row_view;
pub(crate) use views::SkillFactView;

pub(super) const SPARK_DAYS: i64 = 14;
const ROW_LIMIT: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SkillsTab {
    Overview,
    Activity,
    Skills,
}

impl SkillsTab {
    pub(super) const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Skills];

    fn parse(value: Option<&str>) -> AdminResult<Self> {
        Ok(match value.unwrap_or("overview") {
            "overview" => Self::Overview,
            "activity" => Self::Activity,
            "skills" => Self::Skills,
            _ => return Err(AdminError::BadRequest("Unknown skills tab".to_owned())),
        })
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Activity => "activity",
            Self::Skills => "skills",
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Activity => "Activity",
            Self::Skills => "Skills",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillsQuery {
    pub days: Option<i64>,
    pub tab: Option<String>,
    pub marketplace: Option<String>,
    pub client: Option<String>,
    pub search: Option<String>,
    // Why: report links name a skill; it is the search term of the table.
    pub skill: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
}

impl SkillsQuery {
    pub(super) const fn days(&self) -> i64 {
        match self.days {
            Some(7) => 7,
            Some(90) => 90,
            Some(365) => 365,
            _ => 30,
        }
    }

    pub(super) fn trimmed(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
    }

    pub(super) fn marketplace(&self) -> Option<String> {
        Self::trimmed(self.marketplace.as_deref())
    }

    pub(super) fn search(&self) -> Option<String> {
        Self::trimmed(self.search.as_deref()).or_else(|| Self::trimmed(self.skill.as_deref()))
    }

    pub(super) fn sort(&self) -> SkillSort {
        SkillSort::parse_skill_sort(self.sort.as_deref())
    }

    fn tab(&self) -> AdminResult<SkillsTab> {
        SkillsTab::parse(self.tab.as_deref())
    }

    // Why: the page minus the named filters — a chip's remove link and the
    // ribbon's Clear.
    pub(super) fn link_without(&self, tab: SkillsTab, drop: &[&str]) -> String {
        let keep = |name: &str, value: &Option<String>| {
            (!drop.contains(&name)).then(|| value.clone()).flatten()
        };
        let cleared = Self {
            days: self.days,
            tab: self.tab.clone(),
            marketplace: keep("marketplace", &self.marketplace),
            client: keep("client", &self.client),
            search: keep("search", &self.search),
            skill: keep("search", &self.skill),
            sort: keep("sort", &self.sort),
            group: self.group.clone(),
            project: self.project.clone(),
        };
        cleared.link(tab, None, None)
    }

    pub(super) fn link_marketplace(&self, tab: SkillsTab, marketplace: Option<&str>) -> String {
        let scoped = Self {
            days: self.days,
            tab: self.tab.clone(),
            marketplace: marketplace.map(str::to_owned),
            client: self.client.clone(),
            search: self.search.clone(),
            skill: self.skill.clone(),
            sort: self.sort.clone(),
            group: self.group.clone(),
            project: self.project.clone(),
        };
        scoped.link(tab, None, None)
    }

    pub(super) fn link(
        &self,
        tab: SkillsTab,
        days: Option<i64>,
        sort: Option<SkillSort>,
    ) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        if tab != SkillsTab::Overview {
            pairs.push(("tab", tab.as_str().to_owned()));
        }
        let days = days.unwrap_or_else(|| self.days());
        if days != 30 {
            pairs.push(("days", days.to_string()));
        }
        let sort = sort.unwrap_or_else(|| self.sort());
        if sort != SkillSort::Invocations {
            pairs.push(("sort", sort.as_str().to_owned()));
        }
        for (key, value) in [
            ("marketplace", self.marketplace()),
            ("client", Self::trimmed(self.client.as_deref())),
            ("search", self.search()),
            ("group", Self::trimmed(self.group.as_deref())),
            ("project", Self::trimmed(self.project.as_deref())),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        let qs = pairs
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            ANALYSIS_SKILLS_URL.to_owned()
        } else {
            format!("{ANALYSIS_SKILLS_URL}?{qs}")
        }
    }
}

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SkillsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let tab = query.tab()?;
    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let end = Utc::now();
    let window = SkillWindow {
        start: end - Duration::days(query.days()),
        end,
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    };
    let filter = SkillListFilter {
        marketplace_id: query.marketplace().map(MarketplaceId::new),
        search: (tab == SkillsTab::Skills).then(|| query.search()).flatten(),
        client_kind: (tab == SkillsTab::Skills)
            .then(|| SkillsQuery::trimmed(query.client.as_deref()))
            .flatten(),
        sort: query.sort(),
        limit: ROW_LIMIT,
        offset: 0,
        skills: None,
    };
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    let (rows, adoption, audience) = tokio::join!(
        list_skill_facts(&pool, &window, &filter),
        async {
            if tab == SkillsTab::Overview {
                list_marketplace_adoption(&pool, &window).await
            } else {
                Ok(Vec::new())
            }
        },
        get_marketplace_audience(&pool, &index.marketplaces),
    );
    let (rows, adoption, audience) = (rows?, adoption?, audience?);
    let report_banner = crate::handlers::ssr::analysis::reports::banner::report_banner(
        &pool,
        "global",
        None,
        query
            .link(tab, None, None)
            .split_once('?')
            .map_or("", |(_, q)| q),
        "Skills in view",
    )
    .await;
    let context = build_context(
        &query,
        tab,
        &SkillsRead {
            rows: &rows,
            adoption: &adoption,
            audience: &audience,
            index: &index,
        },
        SkillsPageInputs {
            is_admin: shell.user.is_admin,
            report_banner,
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-skills",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
