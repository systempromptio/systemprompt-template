//! Assembling the group listing: the rows, their usage, and the marketplaces
//! each group reaches.
//!
//! Entitlement is resolved through the same resolver the enforcement point
//! uses, against a synthetic subject holding one group membership — so the
//! chips on this page say what a member would actually reach rather than what
//! a rule table happens to mention.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::groups::usage::{GroupUsageRow, UNATTRIBUTED_ROW, list_groups_with_usage};
use crate::services::marketplaces::load_marketplaces;

use super::super::people_view::share;
use super::super::types::{GroupRowView, MemberSetChipView, UnattributedRowView};
use super::UNASSIGNED_GROUP;

pub(super) struct Listing {
    pub rows: Vec<GroupRowView>,
    pub unattributed: UnattributedRowView,
    pub unkeyed_people: i64,
}

pub(super) async fn load_listing(pool: &PgPool, window_days: i32) -> Listing {
    let loaded: Vec<GroupUsageRow> = list_groups_with_usage(pool, window_days)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group usage listing failed"))
        .unwrap_or_default();

    // Why: the remainder arrives as a row of the same partition, so it is
    // split out here rather than queried separately — which is what keeps the
    // rows and the remainder guaranteed to sum to the instance.
    let (remainder, groups): (Vec<GroupUsageRow>, Vec<GroupUsageRow>) =
        loaded.into_iter().partition(|g| g.id == UNATTRIBUTED_ROW);
    let remainder = remainder.into_iter().next();

    let marketplaces = load_group_marketplaces(pool, &groups).await;
    let unkeyed_people = count_unkeyed_people(pool).await;
    let remainder_requests = remainder.as_ref().map_or(0, |r| r.requests);
    let instance_requests: i64 =
        groups.iter().map(|g| g.requests).sum::<i64>() + remainder_requests;

    let rows = groups
        .into_iter()
        .map(|g| GroupRowView {
            href: format!("/admin/groups/{}", g.id),
            is_unassigned: g.id == UNASSIGNED_GROUP,
            marketplaces_title: marketplaces.get(&g.id).map_or_else(String::new, |chips| {
                chips
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            marketplace_count: marketplaces.get(&g.id).map_or(0, Vec::len) as i64,
            project_count: g.project_count,
            member_count: g.member_count,
            active_members_30d: g.active_members,
            requests: g.requests,
            tokens: g.tokens,
            cost_microdollars: g.cost_microdollars,
            top_model_short: g.top_model.as_deref().map(short_model),
            top_model: g.top_model,
            top_model_requests: g.top_model_requests,
            source_abbrev: source_abbrev(source_label(&g.source)),
            source_label: source_label(&g.source),
            source: g.source,
            id: g.id,
            name: g.name,
            description: g.description,
        })
        .collect();

    Listing {
        unattributed: UnattributedRowView {
            requests: remainder_requests,
            tokens: remainder.as_ref().map_or(0, |r| r.tokens),
            cost_microdollars: remainder.as_ref().map_or(0, |r| r.cost_microdollars),
            share_pct: share(remainder_requests, instance_requests),
            people: unkeyed_people,
        },
        unkeyed_people,
        rows,
    }
}

// Why: the listing has ten columns at 1440px and the model name is the widest
// thing that is not a sentence. The vendor prefix is the same on every row of
// an Anthropic estate, so it carries no information here; the cell keeps the
// full id on its title.
fn short_model(model: &str) -> String {
    let tail = model.rsplit('/').next().unwrap_or(model);
    tail.strip_prefix("claude-").unwrap_or(tail).to_owned()
}

// Why: two letters and a title, because the column is one of ten and the word
// itself is never the reason anyone opens this page.
fn source_abbrev(label: &str) -> &'static str {
    match label {
        "Directory" => "Di",
        "System" => "Sy",
        _ => "Da",
    }
}

fn source_label(source: &str) -> &'static str {
    match source {
        "yaml" | "adfs" => "Directory",
        "system" => "System",
        _ => "Dashboard",
    }
}

// Why: How many accounts hold no primary group.
//
// Exclusive attribution has nowhere to file their spend, so it lands in the
// unattributed bucket and every group looks cheaper than it is. The number is
// on the page because the fix — recomputing the keys — is a button beside it,
// and an operator cannot know to press it from a total that merely looks low.
async fn count_unkeyed_people(pool: &PgPool) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
           FROM users u
           WHERE NOT ('anonymous' = ANY(u.roles))
             AND NOT EXISTS (
                 SELECT 1 FROM user_scope_defaults d
                 WHERE d.user_id = u.id AND d.primary_group_id IS NOT NULL)"#
    )
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "unkeyed people count failed"))
    .unwrap_or_default()
}

// Why: Every group's allowed marketplaces, resolved one subject at a time.
//
// The catalog is read once and shared: it comes off disk, and re-reading it
// per group would turn a page render into one filesystem walk per row.
async fn load_group_marketplaces(
    pool: &PgPool,
    groups: &[GroupUsageRow],
) -> HashMap<String, Vec<MemberSetChipView>> {
    let section: Vec<(String, String, Option<String>)> = load_marketplaces()
        .into_iter()
        .map(|m| (m.id.to_string(), m.name, None))
        .collect();
    if section.is_empty() {
        return HashMap::new();
    }

    let subjects: Vec<_> = groups
        .iter()
        .map(|g| repositories::users::access_control::group_subject(&g.id))
        .collect();
    let sections = vec![("marketplace".to_owned(), "Marketplaces".to_owned(), section)];
    let matrices =
        repositories::users::access_control::resolve_subject_matrices(pool, &subjects, &sections)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "group marketplace resolve failed"))
            .unwrap_or_default();
    groups
        .iter()
        .zip(matrices)
        .map(|(group, matrix)| {
            let chips = matrix
                .into_iter()
                .flat_map(|s| s.rows)
                .filter(|r| r.effective == "allow")
                .map(|r| MemberSetChipView {
                    id: r.entity_id,
                    label: r.entity_name,
                })
                .collect();
            (group.id.clone(), chips)
        })
        .collect()
}
