//! The groups plane's reads and its insert-only write, plus the render of
//! the database back as `groups.yaml`.
//!
//! *Overwrite from code* is the boot loader itself
//! ([`crate::repositories::config::groups_yaml_loader`]); only the
//! insert-only direction needs statements of its own, because the loader
//! has no mode that leaves an existing row alone.

use serde::Serialize;
use sqlx::PgPool;

use super::groups_drift::{MappingRow, MemberSetRow};
use crate::repositories::config::groups_yaml_types::GroupsDoc;

pub async fn list_member_sets(pool: &PgPool) -> Result<Vec<MemberSetRow>, sqlx::Error> {
    sqlx::query_as!(
        MemberSetRow,
        r#"SELECT 'group' AS "kind!", id AS "id!", name AS "name!", description,
                  source AS "source!", is_system AS "is_system!"
             FROM groups
            UNION ALL
           SELECT 'project', id, name, description, source, false
             FROM projects
            ORDER BY 1, 2"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_mappings(pool: &PgPool) -> Result<Vec<MappingRow>, sqlx::Error> {
    sqlx::query_as!(
        MappingRow,
        r#"SELECT 'group' AS "kind!", ad_group AS "ad_group!", group_id AS "set_id!",
                  source AS "source!"
             FROM group_ad_mappings
            UNION ALL
           SELECT 'project', ad_group, project_id, source
             FROM project_ad_mappings
            ORDER BY 1, 3, 2"#
    )
    .fetch_all(pool)
    .await
}

pub async fn count_member_sets(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT (SELECT COUNT(*) FROM groups WHERE NOT is_system)
                + (SELECT COUNT(*) FROM projects)
                + (SELECT COUNT(*) FROM group_ad_mappings)
                + (SELECT COUNT(*) FROM project_ad_mappings) AS "count!""#
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InsertOnlyOutcome {
    pub sets: usize,
    pub mappings: usize,
}

// Why: `ON CONFLICT DO NOTHING` throughout — insert-only means a row that
// exists is not so much as touched, whatever its name or source says.
pub async fn insert_missing(
    pool: &PgPool,
    doc: &GroupsDoc,
) -> Result<InsertOnlyOutcome, sqlx::Error> {
    let mut out = InsertOnlyOutcome::default();
    for def in &doc.groups {
        let done = sqlx::query!(
            "INSERT INTO groups (id, name, description, source) VALUES ($1, $2, $3, 'yaml')
             ON CONFLICT (id) DO NOTHING",
            def.id,
            def.name,
            def.description.as_deref()
        )
        .execute(pool)
        .await?;
        out.sets += usize::try_from(done.rows_affected()).unwrap_or(0);
        for ad in &def.ad_groups {
            let done = sqlx::query!(
                "INSERT INTO group_ad_mappings (ad_group, group_id, source) VALUES ($1, $2, 'yaml')
                 ON CONFLICT (ad_group, group_id) DO NOTHING",
                ad,
                def.id
            )
            .execute(pool)
            .await?;
            out.mappings += usize::try_from(done.rows_affected()).unwrap_or(0);
        }
    }
    for def in &doc.projects {
        let done = sqlx::query!(
            "INSERT INTO projects (id, name, description, source) VALUES ($1, $2, $3, 'yaml')
             ON CONFLICT (id) DO NOTHING",
            def.id,
            def.name,
            def.description.as_deref()
        )
        .execute(pool)
        .await?;
        out.sets += usize::try_from(done.rows_affected()).unwrap_or(0);
        for ad in &def.ad_groups {
            let done = sqlx::query!(
                "INSERT INTO project_ad_mappings (ad_group, project_id, source) VALUES ($1, $2, 'yaml')
                 ON CONFLICT (ad_group, project_id) DO NOTHING",
                ad,
                def.id
            )
            .execute(pool)
            .await?;
            out.mappings += usize::try_from(done.rows_affected()).unwrap_or(0);
        }
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
struct ExportSet<'a> {
    id: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    ad_groups: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
struct ExportDoc<'a> {
    groups: Vec<ExportSet<'a>>,
    projects: Vec<ExportSet<'a>>,
}

fn export_sets<'a>(
    kind: &str,
    sets: &'a [MemberSetRow],
    mappings: &'a [MappingRow],
) -> Vec<ExportSet<'a>> {
    sets.iter()
        .filter(|s| s.kind == kind && !s.is_system && s.source != "system")
        .map(|s| ExportSet {
            id: &s.id,
            name: &s.name,
            description: s.description.as_deref().filter(|d| !d.trim().is_empty()),
            ad_groups: mappings
                .iter()
                .filter(|m| m.kind == kind && m.set_id == s.id)
                .map(|m| m.ad_group.as_str())
                .collect(),
        })
        .collect()
}

const EXPORT_HEADER: &str = "# The groups and projects this installation ships with, and the AD groups\n\
# the directory maps into them. Exported from the database by the console:\n\
# every group and project the dashboard holds, system groups excepted, with\n\
# every mapping whatever wrote it. Seeds an empty database at boot and is\n\
# otherwise compared on /admin/sync (groups plane).\n\n";

// Why: The database as `groups.yaml`, in the shape the loader reads.
#[must_use]
pub fn render_groups_export(sets: &[MemberSetRow], mappings: &[MappingRow]) -> String {
    let doc = ExportDoc {
        groups: export_sets("group", sets, mappings),
        projects: export_sets("project", sets, mappings),
    };
    // Why: discard-ok: the export types are plain strings and lists, which
    // cannot fail to serialise; an empty body is the visible fallback
    let body = serde_yaml::to_string(&doc).unwrap_or_default();
    format!("{EXPORT_HEADER}{body}")
}

// Why: The loader's own parse of a file body, for the export round-trip and
// the declaration read.
pub fn parse_groups_doc(yaml: &str) -> Result<GroupsDoc, String> {
    if yaml.trim().is_empty() {
        return Ok(GroupsDoc::default());
    }
    let doc: GroupsDoc = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    doc.validate()?;
    Ok(doc)
}
