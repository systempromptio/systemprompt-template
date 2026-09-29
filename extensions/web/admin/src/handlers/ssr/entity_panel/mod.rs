//! The "Who gets this" panel every catalog detail page carries: who reaches
//! the entity, the rules that decide it, whether code and this database
//! agree about it, and why one named person does or does not reach it.
//!
//! One builder, one partial (`components/entity-access`) and one script
//! (`pages/admin-entity-access.js`), so a marketplace, a plugin, a skill and
//! an MCP server answer the question in the same words and edit it the same
//! way. Every read is best-effort: a part that cannot load leaves its own
//! empty state instead of failing the page it sits on.

mod reach;
mod rules;
pub(crate) mod view;
mod why;

use std::collections::HashMap;

use sqlx::PgPool;

pub(crate) use view::EntityAccessView;
use view::{OptionView, SubjectOptions};

use crate::handlers::ssr::pickable_users::list_pickable_users;
use crate::repositories;
use crate::repositories::access_control::rules::{list_entity_defaults, list_ledger_rules};
use crate::repositories::sync::access_control::{declared_now, drift_now};
use crate::repositories::sync::attention::{ReviewedEntity, reviews_for};
use crate::types::Role;

pub(crate) const SYNC_URL: &str = "/admin/sync?tab=access";

// Why: the page every "access" link lands on — the entity's own panel. A
// kind with no catalog detail page (a gateway route, say) has none.
#[must_use]
pub(crate) fn entity_access_url(entity_type: &str, entity_id: &str) -> Option<String> {
    let base = match entity_type {
        "marketplace" => "/admin/marketplaces",
        "plugin" => "/admin/plugins",
        "skill" => "/admin/skills",
        "mcp_server" => "/admin/mcp",
        "gateway_route" => "/admin/gateway/routes",
        _ => return None,
    };
    Some(format!("{base}/{}#access", urlencoding::encode(entity_id)))
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PanelRequest<'a> {
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub page_url: &'a str,
    pub why: Option<&'a str>,
    pub can_write: bool,
    // Why: a kind whose declaration is not per entity says so above its rules
    // — a gateway route's code rules are the `*` glob, not its own.
    pub note: Option<&'static str>,
}

async fn subject_options(pool: &PgPool) -> SubjectOptions {
    let groups = repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: group listing failed"))
        .unwrap_or_default();
    let projects = repositories::projects::crud::list_project_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: project listing failed"))
        .unwrap_or_default();
    SubjectOptions {
        roles: Role::ALL
            .iter()
            .map(|r| OptionView {
                value: r.as_str().to_owned(),
                label: r.label().to_owned(),
            })
            .collect(),
        groups: groups
            .into_iter()
            .map(|g| OptionView {
                value: g.id.as_str().to_owned(),
                label: g.name,
            })
            .collect(),
        projects: projects
            .into_iter()
            .map(|p| OptionView {
                value: p.id.as_str().to_owned(),
                label: p.name,
            })
            .collect(),
    }
}

fn subject_names(options: &SubjectOptions) -> HashMap<(String, String), String> {
    let mut names = HashMap::new();
    for (band, list) in [
        ("role", &options.roles),
        ("group", &options.groups),
        ("project", &options.projects),
    ] {
        for o in list {
            names.insert((band.to_owned(), o.value.clone()), o.label.clone());
        }
    }
    names
}

// Why: the file failing to load is the panel's own notice, not a 500 — the
// rules that are enforced are still worth reading while someone fixes it.
async fn review_for(pool: &PgPool, key: &str) -> Result<Option<ReviewedEntity>, String> {
    let declared = declared_now().await.map_err(|e| e.to_string())?;
    let drift = drift_now(pool, &declared)
        .await
        .map_err(|e| e.to_string())?;
    let split = reviews_for(pool, &drift).await.map_err(|e| e.to_string())?;
    Ok(split
        .pending
        .into_iter()
        .chain(split.kept)
        .find(|r| r.review.key == key))
}

pub(crate) async fn build_entity_panel(pool: &PgPool, req: PanelRequest<'_>) -> EntityAccessView {
    let key = format!("{}/{}", req.entity_type, req.entity_id);
    let ledger = list_ledger_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: rule listing failed"))
        .unwrap_or_default();
    let rows: Vec<_> = ledger
        .iter()
        .filter(|r| r.entity_type == req.entity_type && r.entity_id == req.entity_id)
        .collect();
    let open = list_entity_defaults(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: default read failed"))
        .unwrap_or_default()
        .into_iter()
        .any(|e| {
            e.entity_type == req.entity_type && e.entity_id == req.entity_id && e.default_included
        });
    let subjects = subject_options(pool).await;
    let mut reaches =
        reach::resolved_reach(pool, (req.entity_type, req.entity_id), &subjects).await;
    reaches.extend(reach::ruled_reach(&rows));
    let (review, drift_unreadable) = match review_for(pool, &key).await {
        Ok(review) => (review, None),
        Err(e) => (None, Some(e)),
    };
    let why = match req.why.map(str::trim).filter(|w| !w.is_empty()) {
        Some(q) => Some(why::why(pool, q, (req.entity_type, req.entity_id)).await),
        None => None,
    };
    EntityAccessView {
        entity_type: req.entity_type.to_owned(),
        entity_id: req.entity_id.to_owned(),
        default_decision: if open { "open" } else { "closed" },
        headline: reach::headline(open, &reaches),
        nobody: !open && reaches.is_empty(),
        reaches,
        bands: rules::band_rules(&rows, &subject_names(&subjects)),
        rule_count: rows.len(),
        not_live: review.as_ref().is_some_and(|r| r.review.not_live),
        review,
        drift_unreadable,
        subjects,
        people: list_pickable_users(pool, None).await,
        can_write: req.can_write,
        note: req.note,
        why,
        page_url: req.page_url.to_owned(),
        sync_url: SYNC_URL,
        key,
    }
}
