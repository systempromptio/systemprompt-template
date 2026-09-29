//! `/admin/analysis/skills/{plugin:skill}` — one skill's record over the
//! window: its KPIs, four day-by-day charts (invocations and people, cost,
//! p95 latency, completion), the invocations split by model, client, group,
//! project, person, marketplace version and outcome, and the conversations
//! that invoked it with the same columns the Conversations page shows, and
//! its timed runs with each kit release's success rate and run time.
//!
//! Invocations are hook events; everything else is read from the
//! `conversation_facts` rollup through the harness session. The skill itself
//! is never a query parameter: its `plugin:skill` key is the path, the same
//! identity the hook events carry.

mod context;
mod figures;
mod runs;
mod views;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::PluginId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::analysis_skill_url;
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::inventory_index::{InventoryIndex, get_marketplace_audience};
use crate::repositories::analysis::skills::{
    SkillBreakdownBy, SkillRunFilter, SkillWindow, find_skill_facts, list_skill_breakdown,
    list_skill_conversations_paged, list_skill_daily, list_skill_runs,
};
use crate::repositories::scope::ScopeRequest;

use context::{SkillRead, build_context};

pub(super) const PAGE_SIZE: i64 = 25;
// Why: the release figures are medians over every run read, so the page reads
// far more runs than it lists; the export has no such cap below its own.
const RUNS_READ: i64 = 1_000;

/// The skill identity the hook events key on, parsed from a `plugin:skill`
/// path segment: the plugin is the prefix, the key is kept whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRef {
    pub plugin_id: PluginId,
    pub skill: String,
}

pub fn parse_skill_key(raw: &str) -> Result<SkillRef, AdminError> {
    let key = raw.trim();
    let bad = key.is_empty()
        || key.len() > 200
        || key.chars().any(|c| c.is_control() || c.is_whitespace());
    let Some((plugin, skill)) = key.split_once(':') else {
        return Err(AdminError::BadRequest(
            "A skill is addressed as plugin:skill".into(),
        ));
    };
    if bad || plugin.is_empty() || skill.is_empty() {
        return Err(AdminError::BadRequest("Invalid skill reference".into()));
    }
    Ok(SkillRef {
        plugin_id: PluginId::new(plugin),
        skill: key.to_owned(),
    })
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillDetailQuery {
    pub days: Option<i64>,
    pub by: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
}

impl SkillDetailQuery {
    pub(super) const fn days(&self) -> i64 {
        match self.days {
            Some(7) => 7,
            Some(90) => 90,
            Some(365) => 365,
            _ => 30,
        }
    }

    pub(super) fn by(&self) -> SkillBreakdownBy {
        match self.by.as_deref() {
            Some("client") => SkillBreakdownBy::Client,
            Some("group") => SkillBreakdownBy::Group,
            Some("project") => SkillBreakdownBy::Project,
            Some("user") => SkillBreakdownBy::User,
            Some("version") => SkillBreakdownBy::Version,
            Some("outcome") => SkillBreakdownBy::Outcome,
            _ => SkillBreakdownBy::Model,
        }
    }

    pub(super) fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    pub(super) fn link(
        &self,
        key: &str,
        days: Option<i64>,
        by: Option<SkillBreakdownBy>,
        page: Option<i64>,
    ) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        let days = days.unwrap_or_else(|| self.days());
        if days != 30 {
            pairs.push(("days", days.to_string()));
        }
        let by = by.unwrap_or_else(|| self.by());
        if by != SkillBreakdownBy::Model {
            pairs.push(("by", by.as_str().to_owned()));
        }
        if let Some(page) = page.filter(|p| *p > 0) {
            pairs.push(("page", page.to_string()));
        }
        for (k, v) in [("group", &self.group), ("project", &self.project)] {
            if let Some(v) = v.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                pairs.push((k, v.to_owned()));
            }
        }
        let qs = pairs
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let base = analysis_skill_url(key);
        if qs.is_empty() {
            base
        } else {
            format!("{base}?{qs}")
        }
    }
}

pub(crate) async fn skill_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(skill): Path<String>,
    Query(query): Query<SkillDetailQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let skill = parse_skill_key(&skill)?;
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
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    let run_filter = SkillRunFilter {
        marketplace: None,
        skill: Some(skill.skill.clone()),
    };
    let (facts, daily, breakdown, conversations, audience, runs) = tokio::join!(
        find_skill_facts(&pool, &window, &skill.skill),
        list_skill_daily(&pool, &window, &skill.skill),
        list_skill_breakdown(&pool, &window, &skill.skill, query.by()),
        list_skill_conversations_paged(
            &pool,
            &window,
            &skill.skill,
            PAGE_SIZE,
            query.page() * PAGE_SIZE
        ),
        get_marketplace_audience(&pool, &index.marketplaces),
        list_skill_runs(&pool, &window, &run_filter, RUNS_READ),
    );
    let (facts, daily, breakdown, (conversations, total), audience, runs) =
        (facts?, daily?, breakdown?, conversations?, audience?, runs?);
    let today = Utc::now().date_naive();
    let row = facts
        .as_ref()
        .map(|f| super::skills::skill_row_view(f, &audience, &index, today));
    let context = build_context(
        &query,
        &skill,
        &index,
        SkillRead {
            row,
            facts: facts.as_ref(),
            daily: &daily,
            breakdown: &breakdown,
            conversations: &conversations,
            runs: &runs,
            total,
            can_judge: shell.user.is_console && !super::judge_mode::automatic(),
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-skill",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
