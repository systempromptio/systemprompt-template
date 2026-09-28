//! Database → `rules.yaml`: the inverse of [`super::declared`].
//!
//! Renders every non-user rule row back into the entity-centric document the
//! loader consumes, so an administrator who edited in the console can export,
//! commit, and have code and database agree again. Output is deterministic
//! (sorted maps, fixed header) so the export diffs cleanly against the file
//! in the repository.
//!
//! Three things cannot round-trip and are said so in the output: a band whose
//! rows carry more than one justification collapses to the entity's `why`,
//! an entity whose rows disagree on `valid_until` is written open-ended, and
//! gateway routes whose rule sets differ from one another collapse to the
//! glob with the exceptions named in a comment, because a literal route id is
//! never a valid declaration.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use systemprompt_security::authz::{Access, EntityKind};

use super::drift::{BandRuleRow, EntityDefaultRow};
use crate::repositories::config::rules_yaml_types::{
    BandMap, BandSpec, EntityDecl, EntityDefault, GLOB_ONLY_KINDS, RulesDoc,
};

const HEADER: &str = "\
# services/access-control/rules.yaml — exported from the database by the console.
# Commit this file to the source repository to make the database state the
# declared state. Per-person (user band) overrides are not exported; they are
# the dashboard's alone. Format and meaning: /documentation/access-control.
";

type Bands = BTreeMap<(String, Access), Vec<(String, Option<String>)>>;

#[derive(Debug, Default)]
struct EntityRows {
    default_included: bool,
    bands: Bands,
    windows: BTreeSet<Option<DateTime<Utc>>>,
}

/// `entity → bundle:<name>` for every entity the declaration marks `owner:`.
///
/// The one fact the database cannot supply, carried across so an export of a
/// kit's marketplace still says where it comes from.
pub type Owners = BTreeMap<String, String>;

#[must_use]
pub fn render_export(
    rules: &[BandRuleRow],
    entities: &[EntityDefaultRow],
    owners: &Owners,
) -> String {
    let mut grouped: BTreeMap<(String, String), EntityRows> = BTreeMap::new();
    for row in rules.iter().filter(|r| r.rule_type != "user") {
        let entry = grouped
            .entry((row.entity_type.clone(), row.entity_id.clone()))
            .or_default();
        entry
            .bands
            .entry((row.rule_type.clone(), row.access))
            .or_default()
            .push((row.rule_value.clone(), row.justification.clone()));
        entry.windows.insert(row.valid_until);
    }
    for entity in entities {
        if let Some(entry) =
            grouped.get_mut(&(entity.entity_type.clone(), entity.entity_id.clone()))
        {
            entry.default_included = entity.default_included;
        }
    }

    let mut notes: Vec<String> = Vec::new();
    let mut doc = RulesDoc::default();
    let mut glob_kinds: BTreeMap<String, Vec<(String, EntityRows)>> = BTreeMap::new();
    for ((entity_type, entity_id), rows) in grouped {
        if GLOB_ONLY_KINDS.iter().any(|k| k.as_str() == entity_type) {
            glob_kinds
                .entry(entity_type)
                .or_default()
                .push((entity_id, rows));
            continue;
        }
        let entity = format!("{entity_type}/{entity_id}");
        let mut decl = decl_for(&entity, &rows, &mut notes);
        decl.owner = owners.get(&entity).cloned();
        doc.entities.push(decl);
    }
    for (entity, owner) in owners {
        if !doc.entities.iter().any(|d| d.entity == *entity) {
            notes.push(format!(
                "{entity}: declared with owner {owner} but its bundle is not active; the entry \
                 in the committed file must be kept by hand"
            ));
        }
    }
    for (entity_type, routes) in glob_kinds {
        collapse_glob(&entity_type, routes, &mut doc, &mut notes);
    }

    let body = serde_yaml::to_string(&doc).unwrap_or_default();
    let mut out = String::from(HEADER);
    for note in notes {
        out.push_str("# NOTE: ");
        out.push_str(&note);
        out.push('\n');
    }
    out.push('\n');
    out.push_str(&body);
    out
}

fn decl_for(entity: &str, rows: &EntityRows, notes: &mut Vec<String>) -> EntityDecl {
    let why = dominant_why(rows).unwrap_or_else(|| {
        notes.push(format!(
            "{entity}: no rule carried a justification; `why` must be written"
        ));
        "TODO: why does this rule exist?".to_owned()
    });
    let mut allow = BandMap::default();
    let mut deny = BandMap::default();
    for ((rule_type, access), values) in &rows.bands {
        let spec = band_spec(entity, rule_type, values, &why, notes);
        match access {
            Access::Allow => allow.set(rule_type, spec),
            Access::Deny => deny.set(rule_type, spec),
        }
    }
    EntityDecl {
        entity: entity.to_owned(),
        default: EntityDefault::from_included(rows.default_included),
        why,
        owner: None,
        valid_until: valid_until_for(entity, rows, notes),
        allow,
        deny,
    }
}

// Why: the declaration has one window per entity, so rows that disagree
// have no single truthful value — the export writes none and says so, and
// the sync then reports those rows as changed until they are brought into
// line by hand.
fn valid_until_for(
    entity: &str,
    rows: &EntityRows,
    notes: &mut Vec<String>,
) -> Option<DateTime<Utc>> {
    match rows.windows.len() {
        0 | 1 => rows.windows.iter().next().copied().flatten(),
        n => {
            notes.push(format!(
                "{entity}: rows carried {n} different valid_until windows; written open-ended"
            ));
            None
        },
    }
}

// Why: a band keeps its own `why` only when every row in it agrees on one
// that differs from the entity's; a band that disagrees with itself has no
// single truthful sentence, so the entity's stands and the export says so.
fn band_spec(
    entity: &str,
    rule_type: &str,
    values: &[(String, Option<String>)],
    entity_why: &str,
    notes: &mut Vec<String>,
) -> BandSpec {
    let mut subjects: Vec<String> = values.iter().map(|(v, _)| v.clone()).collect();
    subjects.sort();
    subjects.dedup();
    let mut whys: Vec<&str> = values.iter().filter_map(|(_, w)| w.as_deref()).collect();
    whys.sort_unstable();
    whys.dedup();
    match whys.as_slice() {
        [one] if *one != entity_why => BandSpec::Detailed {
            values: subjects,
            why: (*one).to_owned(),
        },
        [_, _, ..] => {
            notes.push(format!(
                "{entity}: {rule_type} rows carried {} different justifications; collapsed to the entity why",
                whys.len()
            ));
            BandSpec::List(subjects)
        },
        _ => BandSpec::List(subjects),
    }
}

fn dominant_why(rows: &EntityRows) -> Option<String> {
    let mut tally: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, why) in rows.bands.values().flatten() {
        if let Some(w) = why.as_deref().map(str::trim).filter(|w| !w.is_empty()) {
            *tally.entry(w).or_default() += 1;
        }
    }
    tally
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(w, _)| w.to_owned())
}

// Why: route ids are generated, so the file can only ever say `kind/*`. The
// majority rule set becomes the glob; any route that differs is named in a
// note because the declaration cannot express it and the sync would report
// that route as drift until it is brought into line.
fn collapse_glob(
    entity_type: &str,
    routes: Vec<(String, EntityRows)>,
    doc: &mut RulesDoc,
    notes: &mut Vec<String>,
) {
    let Some(kind) = EntityKind::ALL.iter().find(|k| k.as_str() == entity_type) else {
        return;
    };
    let mut shapes: BTreeMap<String, (Vec<String>, EntityRows)> = BTreeMap::new();
    for (id, rows) in routes {
        let shape = shape_key(&rows);
        let entry = shapes
            .entry(shape)
            .or_insert_with(|| (Vec::new(), EntityRows::default()));
        entry.0.push(id);
        if entry.1.bands.is_empty() {
            entry.1 = rows;
        }
    }
    let Some((_, (majority_ids, majority_rows))) =
        shapes.iter().max_by_key(|(_, (ids, _))| ids.len())
    else {
        return;
    };
    let glob = format!("{}/*", kind.as_str());
    for (_, (ids, _)) in shapes.iter().filter(|(_, (ids, _))| ids != majority_ids) {
        notes.push(format!(
            "{glob}: {} differ from the majority rule set and cannot be declared individually",
            ids.join(", ")
        ));
    }
    doc.entities.push(decl_for(&glob, majority_rows, notes));
}

fn shape_key(rows: &EntityRows) -> String {
    let mut parts: Vec<String> = rows
        .bands
        .iter()
        .map(|((rt, access), values)| {
            let mut v: Vec<&str> = values.iter().map(|(s, _)| s.as_str()).collect();
            v.sort_unstable();
            format!("{rt}:{access}:{}", v.join(","))
        })
        .collect();
    parts.sort();
    let windows: Vec<String> = rows
        .windows
        .iter()
        .map(|w| w.map(|t| t.to_rfc3339()).unwrap_or_default())
        .collect();
    format!(
        "{}|{}|{}",
        rows.default_included,
        parts.join(";"),
        windows.join(",")
    )
}
