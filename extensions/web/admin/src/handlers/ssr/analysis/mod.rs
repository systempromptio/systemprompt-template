//! Analysis uses ingestion facts; the platform catalog remains configuration.
use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analysis::{
    self, AnalysisConversationRow, AnalysisFilter, SkillRow, Totals,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
pub(crate) mod campaigns;
pub(crate) mod candidates;
pub(crate) mod experiments;
pub(crate) mod impact;
pub(crate) mod lifecycle;
pub(crate) mod portfolio;
mod time;
pub(crate) mod versions;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AnalysisQuery {
    #[serde(
        default,
        deserialize_with = "time::deserialize",
        serialize_with = "time::serialize"
    )]
    pub start: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "time::deserialize",
        serialize_with = "time::serialize"
    )]
    pub end: Option<DateTime<Utc>>,
    pub skill: Option<String>,
    pub user: Option<String>,
    pub model: Option<String>,
    pub page: Option<u32>,
}

impl AnalysisQuery {
    fn filter(&self) -> Result<AnalysisFilter, AdminError> {
        let end = self.end.unwrap_or_else(Utc::now);
        let start = self.start.unwrap_or(end - Duration::days(30));
        if start >= end || end - start > Duration::days(366) {
            return Err(AdminError::BadRequest(
                "Choose a window between 1 second and 366 days".into(),
            ));
        }
        if [self.skill.as_ref(), self.user.as_ref(), self.model.as_ref()]
            .into_iter()
            .flatten()
            .any(|s| s.len() > 255 || s.chars().any(char::is_control))
        {
            return Err(AdminError::BadRequest("Invalid analysis filter".into()));
        }
        let clean =
            |value: &Option<String>| value.as_ref().filter(|s| !s.trim().is_empty()).cloned();
        Ok(AnalysisFilter {
            start,
            end,
            skill: clean(&self.skill),
            user: clean(&self.user),
            model: clean(&self.model),
        })
    }

    fn url(&self, page: u32, skill: Option<&str>) -> String {
        let mut pairs = vec![("page".to_owned(), page.to_string())];
        for (key, value) in [
            ("start", self.start.map(|d| d.to_rfc3339())),
            ("end", self.end.map(|d| d.to_rfc3339())),
            ("skill", skill.map(str::to_owned)),
            ("user", self.user.clone()),
            ("model", self.model.clone()),
        ] {
            if let Some(value) = value {
                pairs.push((key.to_owned(), value));
            }
        }
        let query = pairs
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("/admin/analysis/skills?{query}")
    }
}

#[derive(Serialize)]
struct SkillView {
    row: SkillRow,
    href: String,
    cost: String,
}
#[derive(Serialize)]
struct ConversationView {
    row: AnalysisConversationRow,
    href: String,
    cost: String,
}
#[derive(Serialize)]
struct AnalysisPage {
    page: &'static str,
    title: &'static str,
    totals: Totals,
    cost: String,
    filter: AnalysisQuery,
    skills: Vec<SkillView>,
    conversations: Vec<ConversationView>,
    detail: bool,
    previous: Option<String>,
    next: Option<String>,
}
fn dollars(microdollars: i64) -> String {
    super::format::format_cost(microdollars)
}

pub(crate) async fn skills_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(mut query): Query<AnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let filter = query.filter()?;
    query.start = Some(filter.start);
    query.end = Some(filter.end);
    query.skill = filter.skill.clone();
    let page = query.page.unwrap_or(1).max(1);
    let offset = i64::from(page - 1) * 50;
    let totals = analysis::get_totals(&pool, &filter).await?;
    let detail = filter.skill.is_some();
    let mut skills = Vec::new();
    let mut conversations = Vec::new();
    let mut total = 0;
    if detail {
        for row in analysis::list_conversations(&pool, &filter, offset).await? {
            total = row.total_rows;
            conversations.push(ConversationView {
                href: format!(
                    "/admin/sessions/{}",
                    urlencoding::encode(row.session_id.as_str())
                ),
                cost: dollars(row.cost),
                row,
            });
        }
    } else {
        for row in analysis::list_skills(&pool, &filter, offset).await? {
            total = row.total_rows;
            skills.push(SkillView {
                href: query.url(1, Some(&row.skill)),
                cost: dollars(row.cost),
                row,
            });
        }
    }
    let previous = (page > 1).then(|| query.url(page - 1, filter.skill.as_deref()));
    let next = (offset + 50 < total).then(|| query.url(page + 1, filter.skill.as_deref()));
    let context = AnalysisPage {
        page: "analysis-skills",
        title: "Skill analysis",
        cost: dollars(totals.cost),
        totals,
        filter: query,
        skills,
        conversations,
        detail,
        previous,
        next,
    };
    Ok(super::render_typed_page(
        &engine,
        "analysis-skills",
        &context,
        &user,
        &marketplace,
    ))
}
