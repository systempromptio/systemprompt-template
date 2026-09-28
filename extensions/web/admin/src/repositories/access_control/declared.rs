//! The rule set `rules.yaml` says the database should hold.
//!
//! Pure: takes the parsed document plus the two catalogs it needs (the live
//! gateway routes for glob expansion, the marketplace ids for validation) and
//! produces the exact `(entity_type, entity_id, rule_type, rule_value)` rows
//! and entity defaults the seed would write. The drift engine compares this
//! against the database; the sync applies it; the export renders the database
//! back into the document the other way round. Nothing here touches a pool,
//! which is what lets a unit test pin every projection rule.

use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use systemprompt_security::authz::ingestion::glob::glob_matches;
use systemprompt_security::authz::{Access, EntityKind, RegisteredEntities};

use crate::repositories::config::rules_yaml_types::{
    BandSpec, EntityDecl, EntityRef, RulesDoc, access_of,
};

/// Identity of one rule row, in database vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DeclaredKey {
    pub entity_type: String,
    pub entity_id: String,
    pub rule_type: String,
    pub rule_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclaredRule {
    pub key: DeclaredKey,
    pub access: Access,
    pub justification: String,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclaredEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub default_included: bool,
    pub why: String,
}

/// An entity the file declares for a bundle that is not active, so its id
/// cannot be validated yet. Reported, never an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwaitingEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub owner: String,
}

/// The whole declared state, keyed for lookup. `owners` maps `<kind>/<id>`
/// to `bundle:<name>` for every entity declared with an owner, active or
/// awaiting, so the export can write it back.
#[derive(Debug, Default, Clone)]
pub struct DeclaredSet {
    pub entities: BTreeMap<(String, String), DeclaredEntity>,
    pub rules: BTreeMap<DeclaredKey, DeclaredRule>,
    pub awaiting: Vec<AwaitingEntity>,
    pub owners: BTreeMap<String, String>,
}

impl DeclaredSet {
    // Why: the hash of what the file says, independent of formatting and of
    // the catalog it was expanded against. Two operators editing whitespace
    // produce the same hash; a changed reason or subject does not. This is
    // the `declared_hash` the sync state records against every apply.
    #[must_use]
    pub fn declared_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for e in self.entities.values() {
            let open = if e.default_included { "open" } else { "closed" };
            hasher.update(
                format!(
                    "entity\t{}/{}\t{open}\t{}\n",
                    e.entity_type, e.entity_id, e.why
                )
                .as_bytes(),
            );
        }
        for r in self.rules.values() {
            let until = r.valid_until.map(|t| t.to_rfc3339()).unwrap_or_default();
            hasher.update(
                format!(
                    "rule\t{}/{}\t{}\t{}\t{}\t{}\t{until}\n",
                    r.key.entity_type,
                    r.key.entity_id,
                    r.key.rule_type,
                    r.key.rule_value,
                    r.access,
                    r.justification
                )
                .as_bytes(),
            );
        }
        for a in &self.awaiting {
            hasher.update(
                format!("awaiting\t{}/{}\t{}\n", a.entity_type, a.entity_id, a.owner).as_bytes(),
            );
        }
        hex::encode(hasher.finalize())
    }

    // Why: Whether the file speaks for this entity at all. A declared entity
    // owns every non-user band on it, so a database row at any of those bands
    // that the file does not list is drift; an undeclared entity's rows are
    // the dashboard's alone and are only ever reported, never deleted.
    #[must_use]
    pub fn governs(&self, entity_type: &str, entity_id: &str) -> bool {
        self.entities
            .contains_key(&(entity_type.to_owned(), entity_id.to_owned()))
    }

    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    // Why: a declaration whose window has closed is not a declaration. Kept
    // out of the pure projection so a test can pin what the file says; the
    // loader applies it once with the clock it has, and the entity stays
    // governed so the rows the sweep removes are never reported as drift.
    pub fn drop_expired(&mut self, now: DateTime<Utc>) {
        self.rules
            .retain(|_, rule| rule.valid_until.is_none_or(|until| until > now));
    }
}

#[derive(Debug)]
pub enum DeclaredError {
    UnknownMarketplace(String),
    UnregisteredEntity(String),
    // Why: a glob that names no entity declares nothing — applied, it would
    // write no rule and report no drift, and the routes it meant to open stay
    // closed behind a clean sync card.
    EmptyGlob(String),
    Invalid(String),
}

impl fmt::Display for DeclaredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMarketplace(id) => write!(
                f,
                "rules.yaml names marketplace/{id}, which services/marketplaces/ does not define"
            ),
            Self::EmptyGlob(entity) => write!(
                f,
                "rules.yaml declares {entity}, but the live catalog holds no entity of that kind \
                 for the glob to expand over — the gateway has no dispatchable routes, or the \
                 services tree did not compose"
            ),
            Self::UnregisteredEntity(msg) | Self::Invalid(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for DeclaredError {}

/// Catalogs the projection consults.
///
/// All data, so tests need no database: every `gateway_route` id the live
/// gateway configuration dispatches (never the database — a truncated or
/// not-yet-bootstrapped `access_control_entities` must not shrink what the
/// file declares), every marketplace id the composed services tree defines,
/// and the registered entities whose literal ids must already exist.
#[derive(Debug)]
pub struct DeclaredInputs<'a> {
    pub gateway_routes: &'a [String],
    pub marketplace_ids: &'a [String],
    pub registered: &'a RegisteredEntities,
}

pub fn build_declared_set(
    doc: &RulesDoc,
    inputs: &DeclaredInputs<'_>,
) -> Result<DeclaredSet, DeclaredError> {
    let mut set = DeclaredSet::default();
    for decl in &doc.entities {
        let entity: EntityRef = decl.entity.parse().map_err(DeclaredError::Invalid)?;
        if let Some(owner) = &decl.owner {
            set.owners.insert(entity.to_string(), owner.clone());
        }
        let ids = match expand_ids(&entity, inputs) {
            Ok(ids) => ids,
            // Why: an id a bundle owns is allowed to be missing until that
            // bundle is active — access for a kit's marketplace is declared
            // before its digest is pinned, and the page says so instead of
            // the boot failing.
            Err(DeclaredError::UnknownMarketplace(_) | DeclaredError::UnregisteredEntity(_))
                if decl.owner.is_some() =>
            {
                set.awaiting.push(AwaitingEntity {
                    entity_type: entity.kind.as_str().to_owned(),
                    entity_id: entity.id.clone(),
                    owner: decl.owner.clone().unwrap_or_default(),
                });
                continue;
            },
            Err(e) => return Err(e),
        };
        for id in ids {
            project_entity(&mut set, &entity.kind, &id, decl);
        }
    }
    Ok(set)
}

fn expand_ids(
    entity: &EntityRef,
    inputs: &DeclaredInputs<'_>,
) -> Result<Vec<String>, DeclaredError> {
    if entity.is_glob() {
        let ids: Vec<String> = match entity.kind {
            EntityKind::GatewayRoute => inputs
                .gateway_routes
                .iter()
                .filter(|id| glob_matches(&entity.id, id))
                .cloned()
                .collect(),
            _ => Vec::new(),
        };
        if ids.is_empty() {
            return Err(DeclaredError::EmptyGlob(entity.to_string()));
        }
        return Ok(ids);
    }
    if entity.kind == EntityKind::Marketplace && !inputs.marketplace_ids.contains(&entity.id) {
        return Err(DeclaredError::UnknownMarketplace(entity.id.clone()));
    }
    inputs
        .registered
        .require(entity.kind, &entity.id)
        .map_err(|e| DeclaredError::UnregisteredEntity(e.to_string()))?;
    Ok(vec![entity.id.clone()])
}

fn project_entity(set: &mut DeclaredSet, kind: &EntityKind, id: &str, decl: &EntityDecl) {
    let entity_type = kind.as_str().to_owned();
    set.entities.insert(
        (entity_type.clone(), id.to_owned()),
        DeclaredEntity {
            entity_type: entity_type.clone(),
            entity_id: id.to_owned(),
            default_included: decl.default.included(),
            why: decl.why.trim().to_owned(),
        },
    );
    for (is_allow, bands) in [(true, &decl.allow), (false, &decl.deny)] {
        for band in bands.bands() {
            project_band(set, (&entity_type, id), band, access_of(is_allow), decl);
        }
    }
}

fn project_band(
    set: &mut DeclaredSet,
    (entity_type, entity_id): (&str, &str),
    (rule_type, spec): (&str, &BandSpec),
    access: Access,
    decl: &EntityDecl,
) {
    let why = spec.why().unwrap_or(&decl.why).trim().to_owned();
    for value in spec.values() {
        let key = DeclaredKey {
            entity_type: entity_type.to_owned(),
            entity_id: entity_id.to_owned(),
            rule_type: rule_type.to_owned(),
            rule_value: value.trim().to_owned(),
        };
        set.rules.insert(
            key.clone(),
            DeclaredRule {
                key,
                access,
                justification: why.clone(),
                valid_until: decl.valid_until,
            },
        );
    }
}
