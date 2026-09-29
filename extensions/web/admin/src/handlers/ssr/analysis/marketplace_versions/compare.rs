//! The Compare view: two versions of one marketplace side by side — headline
//! figures for each, then every skill either version carried with its change
//! and its figures under each version.

use std::collections::BTreeMap;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SkillId};

use super::diff::{Change, diff};
use super::history::{VersionView, cost, latency, short, views};
use crate::error::{AdminError, AdminResult};
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceVersionMetricsRow, MarketplaceVersionSkillRow, VersionCompletionRow, VersionWindow,
    list_marketplace_version_skills,
};

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct SkillFigures {
    pub present: bool,
    pub invocations: i64,
    pub users: i64,
    pub requests: i64,
    pub failed: i64,
    pub cost: String,
    pub p95: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillCompareRow {
    pub plugin_id: PluginId,
    pub skill_id: SkillId,
    pub skill_key: String,
    pub change: Change,
    pub change_label: &'static str,
    pub change_tone: &'static str,
    pub a: SkillFigures,
    pub b: SkillFigures,
    pub invocations_delta: i64,
    pub failed_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompareView {
    pub sides: [VersionView; 2],
    pub options: Vec<HashOption>,
    pub skills: Vec<SkillCompareRow>,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub comparable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HashOption {
    pub hash: String,
    pub label: String,
    pub is_a: bool,
    pub is_b: bool,
}

fn figures(row: Option<&MarketplaceVersionSkillRow>) -> SkillFigures {
    row.map_or_else(SkillFigures::default, |r| SkillFigures {
        present: true,
        invocations: r.invocations,
        users: r.users,
        requests: r.requests,
        failed: r.failed,
        cost: cost(r.cost),
        p95: latency(r.p95_ms, r.latency_measured),
    })
}

// Why: `a` is the older side and `b` the newer. With neither named, the
// two newest versions are compared, which is the question a reader most
// often arrives with: did the latest change help.
fn pick(
    rows: &[MarketplaceVersionMetricsRow],
    (a, b): (Option<String>, Option<String>),
) -> AdminResult<(&MarketplaceVersionMetricsRow, &MarketplaceVersionMetricsRow)> {
    let find = |hash: &str| {
        rows.iter()
            .find(|r| r.content_hash == hash)
            .ok_or_else(|| AdminError::NotFound(format!("No version {hash} of this marketplace")))
    };
    let (Some(newest), older) = (rows.first(), rows.get(1)) else {
        return Err(AdminError::NotFound("No version recorded".to_owned()));
    };
    let b = b.as_deref().map_or(Ok(newest), find)?;
    let a = a
        .as_deref()
        .map_or_else(|| Ok(older.unwrap_or(newest)), find)?;
    Ok((a, b))
}

type ByKey<'a> = BTreeMap<(&'a str, &'a str, &'a str), &'a MarketplaceVersionSkillRow>;

// Why: a skill is listed if either manifest names it or either version has
// facts for it — a legacy version has no manifest, so its skills are known
// only from its facts.
fn skill_rows_for(
    manifest_diff: Option<&super::diff::ManifestDiff>,
    skill_rows: &[MarketplaceVersionSkillRow],
    by_key: &ByKey<'_>,
    (a, b): (&MarketplaceVersionMetricsRow, &MarketplaceVersionMetricsRow),
) -> Vec<SkillCompareRow> {
    let mut keys: BTreeMap<(PluginId, String), (SkillId, Change)> = BTreeMap::new();
    if let Some(d) = manifest_diff {
        for s in &d.skills {
            keys.insert(
                (s.plugin_id.clone(), s.skill_key.clone()),
                (s.skill_id.clone(), s.change),
            );
        }
    }
    for r in skill_rows {
        keys.entry((r.plugin_id.clone(), r.skill.clone()))
            .or_insert_with(|| (SkillId::new(r.skill.clone()), Change::Unchanged));
    }
    keys.into_iter()
        .map(|((plugin_id, skill_key), (skill_id, change))| {
            let lookup = |hash: &str| {
                by_key
                    .get(&(hash, plugin_id.as_str(), skill_key.as_str()))
                    .copied()
            };
            let fa = figures(lookup(&a.content_hash));
            let fb = figures(lookup(&b.content_hash));
            SkillCompareRow {
                invocations_delta: fb.invocations - fa.invocations,
                failed_delta: fb.failed - fa.failed,
                plugin_id,
                skill_id,
                skill_key,
                change,
                change_label: change.label(),
                change_tone: change.tone(),
                a: fa,
                b: fb,
            }
        })
        .collect()
}

// Why: every version of one marketplace with the judge's scores beside it —
// what the detail page has already read once and every tab reads from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Recorded<'a> {
    pub rows: &'a [MarketplaceVersionMetricsRow],
    pub scores: &'a [VersionCompletionRow],
}

pub(crate) async fn build(
    pool: &PgPool,
    window: VersionWindow,
    recorded: Recorded<'_>,
    hashes: (Option<String>, Option<String>),
) -> AdminResult<CompareView> {
    let Recorded { rows, scores } = recorded;
    let (a, b) = pick(rows, hashes)?;
    let wanted = [a.content_hash.clone(), b.content_hash.clone()];
    let skill_rows =
        list_marketplace_version_skills(pool, window, &a.marketplace_id, &wanted).await?;
    let by_key: ByKey<'_> = skill_rows
        .iter()
        .map(|r| {
            (
                (
                    r.marketplace_hash.as_str(),
                    r.plugin_id.as_str(),
                    r.skill.as_str(),
                ),
                r,
            )
        })
        .collect();

    let manifest_diff = match (a.manifest.as_ref(), b.manifest.as_ref()) {
        (older, Some(newer)) => Some(diff(older.map(|m| &m.0), &newer.0)),
        _ => None,
    };
    let comparable = manifest_diff.is_some();

    let skills = skill_rows_for(manifest_diff.as_ref(), &skill_rows, &by_key, (a, b));

    let pair = [a.clone(), b.clone()];
    let mut both = views(&pair, scores);
    let b_view = both
        .pop()
        .ok_or_else(|| AdminError::internal("compare view"))?;
    let a_view = both
        .pop()
        .ok_or_else(|| AdminError::internal("compare view"))?;
    Ok(CompareView {
        options: rows
            .iter()
            .map(|r| HashOption {
                label: format!(
                    "{}{}",
                    short(&r.content_hash),
                    if r.effective_until.is_none() {
                        " (current)"
                    } else {
                        ""
                    }
                ),
                is_a: r.content_hash == a.content_hash,
                is_b: r.content_hash == b.content_hash,
                hash: r.content_hash.clone(),
            })
            .collect(),
        added: manifest_diff.as_ref().map_or(0, |d| d.added),
        removed: manifest_diff.as_ref().map_or(0, |d| d.removed),
        changed: manifest_diff.as_ref().map_or(0, |d| d.changed),
        unchanged: manifest_diff.as_ref().map_or(0, |d| d.unchanged),
        comparable,
        sides: [a_view, b_view],
        skills,
    })
}
