//! Bootstrap loader: `services/web/config/groups.yaml` → DB.
//!
//! Upserts the groups and projects the installation ships with, and the AD
//! group mappings that place a signing-in user in them. Rows written here
//! carry `source = 'yaml'`; dashboard-created rows carry `source =
//! 'dashboard'` and are never touched.
//!
//! Reconciliation is deliberately asymmetric. Mappings are fully reconciled —
//! a mapping dropped from the file is dropped from the DB, or a group removed
//! from the directory would keep enrolling people forever. Groups and projects
//! themselves are only ever upserted: they own membership, usage history and
//! access rules, and deleting one because an editor removed six lines from a
//! YAML file would cascade all of that away. Removing a group is a dashboard
//! act, taken deliberately, with the member list in front of you.

use std::path::Path;

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use super::groups_yaml_types::{GroupsDoc, GroupsLoadReport, MemberSetDef};

const GROUPS_FILE: &str = "web/config/groups.yaml";

pub async fn load_groups_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<GroupsLoadReport, MarketplaceError> {
    let mut report = GroupsLoadReport::default();
    let Some(doc) = read_doc(services_path).await? else {
        return Ok(report);
    };
    doc.validate()
        .map_err(|e| MarketplaceError::config_file(GROUPS_FILE, e))?;

    for def in &doc.groups {
        upsert_group(pool, def).await?;
        report.groups += 1;
    }
    for def in &doc.projects {
        upsert_project(pool, def).await?;
        report.projects += 1;
    }

    report.mappings = reconcile_group_mappings(pool, &doc.groups).await?
        + reconcile_project_mappings(pool, &doc.projects).await?;

    tracing::info!(
        groups = report.groups,
        projects = report.projects,
        mappings = report.mappings,
        "bootstrap_groups_loaded"
    );
    Ok(report)
}

async fn read_doc(services_path: &Path) -> Result<Option<GroupsDoc>, MarketplaceError> {
    let path = services_path.join(GROUPS_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(GroupsDoc::default())),
        Ok(s) => serde_yaml::from_str::<GroupsDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(GROUPS_FILE, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

// Why: name and description only. `source` is set on insert and left alone on
// conflict, so a group an operator created in the dashboard and later added to
// the file keeps its dashboard provenance rather than becoming YAML-owned and
// exposed to the mapping reconciliation below.
//
// Why: groups and projects get one function each rather than one function
// taking the table name. A statement the `sqlx` macros verify has to be a
// literal, and a table name threaded through a parameter is exactly the shape
// that keeps a query out of the compiler's reach.
async fn upsert_group(pool: &PgPool, def: &MemberSetDef) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO groups (id, name, description, source)
         VALUES ($1, $2, $3, 'yaml') ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name,
         description = EXCLUDED.description, updated_at = CURRENT_TIMESTAMP",
        def.id,
        def.name,
        def.description.as_deref()
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_project(pool: &PgPool, def: &MemberSetDef) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO projects (id, name, description, source)
         VALUES ($1, $2, $3, 'yaml') ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name,
         description = EXCLUDED.description, updated_at = CURRENT_TIMESTAMP",
        def.id,
        def.name,
        def.description.as_deref()
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: the mappings arrive as one row per AD group per member set, so the
// pairs are flattened into two parallel arrays and unnested back into rows.
// One round trip whatever the file's size.
fn mapping_pairs(defs: &[MemberSetDef]) -> (Vec<String>, Vec<String>) {
    let mut ad_groups = Vec::new();
    let mut ids = Vec::new();
    for def in defs {
        for ad_group in &def.ad_groups {
            ad_groups.push(ad_group.clone());
            ids.push(def.id.clone());
        }
    }
    (ad_groups, ids)
}

async fn reconcile_group_mappings(
    pool: &PgPool,
    defs: &[MemberSetDef],
) -> Result<usize, MarketplaceError> {
    let (ad_groups, ids) = mapping_pairs(defs);
    sqlx::query!(
        "INSERT INTO group_ad_mappings (ad_group, group_id, source)
         SELECT a, i, 'yaml' FROM UNNEST($1::TEXT[], $2::TEXT[]) AS m(a, i)
         ON CONFLICT (ad_group, group_id) DO NOTHING",
        &ad_groups,
        &ids
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        "DELETE FROM group_ad_mappings WHERE source = 'yaml'
         AND (ad_group, group_id) NOT IN
         (SELECT a, i FROM UNNEST($1::TEXT[], $2::TEXT[]) AS kept(a, i))",
        &ad_groups,
        &ids
    )
    .execute(pool)
    .await?;
    Ok(ad_groups.len())
}

async fn reconcile_project_mappings(
    pool: &PgPool,
    defs: &[MemberSetDef],
) -> Result<usize, MarketplaceError> {
    let (ad_groups, ids) = mapping_pairs(defs);
    sqlx::query!(
        "INSERT INTO project_ad_mappings (ad_group, project_id, source)
         SELECT a, i, 'yaml' FROM UNNEST($1::TEXT[], $2::TEXT[]) AS m(a, i)
         ON CONFLICT (ad_group, project_id) DO NOTHING",
        &ad_groups,
        &ids
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        "DELETE FROM project_ad_mappings WHERE source = 'yaml'
         AND (ad_group, project_id) NOT IN
         (SELECT a, i FROM UNNEST($1::TEXT[], $2::TEXT[]) AS kept(a, i))",
        &ad_groups,
        &ids
    )
    .execute(pool)
    .await?;
    Ok(ad_groups.len())
}
