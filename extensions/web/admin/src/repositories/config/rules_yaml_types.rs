//! Wire types for `services/access-control/rules.yaml`, the one declarative
//! source of entitlement on this instance.
//!
//! The file is entity-centric: one block per entity, each band under `allow`
//! or `deny` listing the subjects it names. The band keys are the database
//! `rule_type` values verbatim, which is what lets the export module render a
//! database back into this exact shape. The same types serialise, so a
//! round-trip (parse → project → export → parse) is the identity.
//!
//! `why` is mandatory per entity. It lands on every rule row of that entity
//! as its `justification`, so the ledger can finally say why a rule exists
//! instead of leaving the column blank.
//!
//! `BandMap::bands` yields ladder order, narrowest first — the order the
//! resolver consults. Glob-only kinds have generated ids: a literal id of one
//! names a row that cannot exist, so only the glob is accepted. An `owner:`
//! value is `bundle:` plus a source name from the profile's
//! `services.sources[]`.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_security::authz::{Access, EntityKind};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesDoc {
    #[serde(default)]
    pub entities: Vec<EntityDecl>,
}

/// One entity's declared access, keyed by `entity` = `<kind>/<id>` (or
/// `<kind>/*` for the glob-only kinds).
///
/// `owner` is `bundle:<name>` when the entity arrives in a remote services
/// bundle rather than this tree: access is still declared here — a kit never
/// widens its own audience — but the id may be absent until that bundle is
/// active, and is reported as *awaiting its bundle*.
///
/// `valid_until` is an RFC 3339 instant after which every rule of the entity
/// stops applying. It lands on each rule row's validity record; the hourly
/// expiry sweep deletes the rows once it passes, and the loader treats a
/// declaration already past it as not declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityDecl {
    pub entity: String,
    #[serde(default)]
    pub default: EntityDefault,
    pub why: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "BandMap::is_empty")]
    pub allow: BandMap,
    #[serde(default, skip_serializing_if = "BandMap::is_empty")]
    pub deny: BandMap,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityDefault {
    Open,
    #[default]
    Closed,
}

impl EntityDefault {
    #[must_use]
    pub const fn included(self) -> bool {
        matches!(self, Self::Open)
    }

    #[must_use]
    pub const fn from_included(included: bool) -> Self {
        if included { Self::Open } else { Self::Closed }
    }
}

/// The subject bands this instance writes rules at. `user` is absent on
/// purpose: per-person overrides are dashboard-only and never declared.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BandMap {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector: Option<BandSpec>,
}

impl BandMap {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.role.is_none()
            && self.group.is_none()
            && self.project.is_none()
            && self.connector.is_none()
    }

    #[must_use]
    pub fn bands(&self) -> Vec<(&'static str, &BandSpec)> {
        [
            ("project", &self.project),
            ("group", &self.group),
            ("connector", &self.connector),
            ("role", &self.role),
        ]
        .into_iter()
        .filter_map(|(name, spec)| spec.as_ref().map(|s| (name, s)))
        .collect()
    }

    pub fn set(&mut self, rule_type: &str, spec: BandSpec) {
        match rule_type {
            "role" => self.role = Some(spec),
            "group" => self.group = Some(spec),
            "project" => self.project = Some(spec),
            "connector" => self.connector = Some(spec),
            _ => {},
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BandSpec {
    List(Vec<String>),
    Detailed { values: Vec<String>, why: String },
}

impl BandSpec {
    #[must_use]
    pub fn values(&self) -> &[String] {
        match self {
            Self::List(v) | Self::Detailed { values: v, .. } => v,
        }
    }

    #[must_use]
    pub fn why(&self) -> Option<&str> {
        match self {
            Self::List(_) => None,
            Self::Detailed { why, .. } => Some(why),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRef {
    pub kind: EntityKind,
    pub id: String,
}

impl EntityRef {
    #[must_use]
    pub fn is_glob(&self) -> bool {
        self.id.contains('*')
    }
}

impl FromStr for EntityRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, id) = s
            .split_once('/')
            .ok_or_else(|| format!("entity '{s}' must be written as <kind>/<id>"))?;
        if id.trim().is_empty() {
            return Err(format!("entity '{s}' names no id"));
        }
        let kind = EntityKind::from_str(kind).map_err(|e| e.to_string())?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.kind.as_str(), self.id)
    }
}

pub const OWNER_BUNDLE_PREFIX: &str = "bundle:";

impl EntityDecl {
    #[must_use]
    pub fn owner_bundle(&self) -> Option<&str> {
        self.owner.as_deref()?.strip_prefix(OWNER_BUNDLE_PREFIX)
    }

    fn validate(&self) -> Result<(), String> {
        let entity: EntityRef = self.entity.parse()?;
        if self.why.trim().is_empty() {
            return Err(format!("{entity}: `why` is required"));
        }
        check_glob_shape(&entity)?;
        if self.allow.is_empty() && self.deny.is_empty() {
            return Err(format!("{entity}: declares no allow and no deny"));
        }
        self.check_owner(&entity)?;
        if let Some((band, subject)) = self.first_conflict() {
            return Err(format!(
                "{entity}: {band} '{subject}' is both allowed and denied"
            ));
        }
        self.check_band_subjects(&entity)
    }

    fn check_owner(&self, entity: &EntityRef) -> Result<(), String> {
        let Some(owner) = self.owner.as_deref() else {
            return Ok(());
        };
        match self.owner_bundle() {
            Some(name) if !name.trim().is_empty() => {},
            _ => {
                return Err(format!(
                    "{entity}: owner '{owner}' must be written as {OWNER_BUNDLE_PREFIX}<name>"
                ));
            },
        }
        if entity.is_glob() {
            return Err(format!("{entity}: a glob cannot name an owner"));
        }
        Ok(())
    }

    fn first_conflict(&self) -> Option<(&'static str, &str)> {
        let denied = self.deny.bands();
        self.allow.bands().into_iter().find_map(|(band, allowed)| {
            let (_, denied) = denied.iter().find(|(b, _)| *b == band)?;
            allowed
                .values()
                .iter()
                .find(|v| denied.values().contains(v))
                .map(|v| (band, v.as_str()))
        })
    }

    fn check_band_subjects(&self, entity: &EntityRef) -> Result<(), String> {
        let bands = self.allow.bands().into_iter().chain(self.deny.bands());
        for (band, spec) in bands {
            if let Some(problem) = subject_problem(spec) {
                return Err(format!("{entity}: {band} {problem}"));
            }
        }
        Ok(())
    }
}

fn check_glob_shape(entity: &EntityRef) -> Result<(), String> {
    let kind = entity.kind.as_str();
    match (entity.is_glob(), GLOB_ONLY_KINDS.contains(&entity.kind)) {
        (true, false) => Err(format!(
            "{entity}: only gateway_route and hook take a glob; write the id"
        )),
        (false, true) => Err(format!(
            "{entity}: {kind} ids are generated, never written — use {kind}/*"
        )),
        (true, true) | (false, false) => Ok(()),
    }
}

fn subject_problem(spec: &BandSpec) -> Option<&'static str> {
    let values = spec.values();
    if values.is_empty() {
        Some("names no subjects")
    } else if values.iter().any(|v| v.trim().is_empty()) {
        Some("contains a blank subject")
    } else {
        None
    }
}

pub const BAND_NAMES: [&str; 4] = ["role", "group", "project", "connector"];

pub const GLOB_ONLY_KINDS: [EntityKind; 2] = [EntityKind::GatewayRoute, EntityKind::Hook];

impl RulesDoc {
    pub fn validate(&self) -> Result<(), String> {
        self.entities.iter().try_for_each(EntityDecl::validate)
    }
}

#[must_use]
pub const fn access_of(band_is_allow: bool) -> Access {
    if band_is_allow {
        Access::Allow
    } else {
        Access::Deny
    }
}
