//! The access-control drift regrouped by entity, one reviewable row each.
//!
//! [`super::drift`] lists disagreements rule by rule; a person settles them
//! entity by entity. Each entity gets one [`ReviewKind`], chosen by what the
//! database can and cannot know:
//!
//! - *New in code* — the file names an entity this database has never
//!   catalogued, so it was never applied. Nobody reaches it through code until
//!   someone applies it.
//! - *Removed in console* — the entity is live but lacks a rule the file
//!   declares. The database cannot tell a console removal from a rule added to
//!   code later, so the wording names the likelier cause and the diff shows
//!   both sides.
//! - *Changed in code* — both hold the row and disagree, or code stopped
//!   declaring a row it once wrote.
//! - *Only in console* — the database holds rows the file never declared.
//!
//! A kit's own rows on an entity the file does not govern are the kit's
//! truth and never reach the review. The fingerprint is the entity's diff
//! hashed, so a "keep the database" decision holds until either side moves.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::drift::{DriftReport, OrphanOrigin};
use crate::repositories::sync::access_control_rows::band_label;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    NewInCode,
    RemovedInConsole,
    ChangedInCode,
    OnlyInConsole,
}

impl ReviewKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NewInCode => "New in code",
            Self::RemovedInConsole => "Removed in console",
            Self::ChangedInCode => "Changed in code",
            Self::OnlyInConsole => "Only in console",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::NewInCode => "err",
            Self::RemovedInConsole | Self::ChangedInCode => "warn",
            Self::OnlyInConsole => "info",
        }
    }

    #[must_use]
    pub const fn advice(self) -> &'static str {
        match self {
            Self::NewInCode => {
                "Declared in code, not live. Nobody reaches it through these rules until it is applied."
            },
            Self::RemovedInConsole => {
                "Live, but missing a rule code declares — most likely removed in the console. Apply code restores it; keep the database and export to make the removal permanent."
            },
            Self::ChangedInCode => {
                "Code and this database hold different versions of the same rules. Apply code takes the file's; keep the database and export to take this one."
            },
            Self::OnlyInConsole => {
                "Written in the console and not in code. Export to carry it into the file."
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewLine {
    pub band: String,
    pub band_label: &'static str,
    pub subject: String,
    pub in_code: String,
    pub in_db: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityReview {
    pub entity_type: String,
    pub entity_id: String,
    pub key: String,
    pub kind: ReviewKind,
    pub kind_label: &'static str,
    pub kind_tone: &'static str,
    pub advice: &'static str,
    pub not_live: bool,
    // Why: an entity-scoped overwrite keeps console rows on an entity the
    // file does not govern, so "Apply code" would move nothing there.
    pub applies: bool,
    pub lines: Vec<ReviewLine>,
    pub fingerprint: String,
}

#[derive(Default)]
struct Acc {
    new_entity: bool,
    missing: bool,
    changed: bool,
    applies: bool,
    lines: Vec<ReviewLine>,
}

fn why(access: impl std::fmt::Display, why: Option<&str>) -> String {
    format!("{access} — {}", why.unwrap_or("(no reason recorded)"))
}

fn line(band: &str, subject: &str, in_code: String, in_db: String) -> ReviewLine {
    ReviewLine {
        band: band.to_owned(),
        band_label: if band.is_empty() {
            "default"
        } else {
            band_label(band)
        },
        subject: subject.to_owned(),
        in_code,
        in_db,
    }
}

const fn open_word(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

fn at<'a>(by: &'a mut BTreeMap<(String, String), Acc>, t: &str, i: &str) -> &'a mut Acc {
    by.entry((t.to_owned(), i.to_owned())).or_default()
}

fn collect(drift: &DriftReport) -> BTreeMap<(String, String), Acc> {
    let mut by: BTreeMap<(String, String), Acc> = BTreeMap::new();
    for e in &drift.entities_missing {
        let acc = at(&mut by, &e.entity_type, &e.entity_id);
        acc.new_entity = true;
        acc.applies = true;
        let code = open_word(e.default_included).to_owned();
        acc.lines
            .push(line("", "(entity default)", code, "—".to_owned()));
    }
    for r in &drift.missing_in_db {
        let acc = at(&mut by, &r.key.entity_type, &r.key.entity_id);
        acc.missing = true;
        acc.applies = true;
        let code = why(r.access, Some(&r.justification));
        acc.lines.push(line(
            &r.key.rule_type,
            &r.key.rule_value,
            code,
            "—".to_owned(),
        ));
    }
    for c in &drift.changed {
        let acc = at(&mut by, &c.key.entity_type, &c.key.entity_id);
        acc.changed = true;
        acc.applies = true;
        let code = why(c.declared_access, Some(&c.declared_why));
        let db = why(c.db_access, c.db_why.as_deref());
        acc.lines
            .push(line(&c.key.rule_type, &c.key.rule_value, code, db));
    }
    for d in &drift.default_changed {
        let acc = at(&mut by, &d.entity_type, &d.entity_id);
        acc.changed = true;
        acc.applies = true;
        let code = open_word(d.declared_open).to_owned();
        let db = open_word(d.db_open).to_owned();
        acc.lines.push(line("", "(entity default)", code, db));
    }
    for o in &drift.only_in_db {
        if o.origin == OrphanOrigin::Bundle && !o.governed {
            continue;
        }
        let acc = at(&mut by, &o.row.entity_type, &o.row.entity_id);
        acc.changed |= o.origin == OrphanOrigin::Code;
        acc.applies |= o.retire;
        let db = why(o.row.access, o.row.justification.as_deref());
        acc.lines.push(line(
            &o.row.rule_type,
            &o.row.rule_value,
            "—".to_owned(),
            db,
        ));
    }
    by
}

fn fingerprint(kind: ReviewKind, lines: &[ReviewLine]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.label().as_bytes());
    for l in lines {
        hasher
            .update(format!("\n{}\t{}\t{}\t{}", l.band, l.subject, l.in_code, l.in_db).as_bytes());
    }
    hex::encode(hasher.finalize())[..16].to_owned()
}

#[must_use]
pub fn review_entities(drift: &DriftReport) -> Vec<EntityReview> {
    let mut out: Vec<EntityReview> = collect(drift)
        .into_iter()
        .map(|((entity_type, entity_id), acc)| {
            let kind = if acc.new_entity {
                ReviewKind::NewInCode
            } else if acc.missing {
                ReviewKind::RemovedInConsole
            } else if acc.changed {
                ReviewKind::ChangedInCode
            } else {
                ReviewKind::OnlyInConsole
            };
            let mut lines = acc.lines;
            lines.sort_by(|a, b| (&a.band, &a.subject).cmp(&(&b.band, &b.subject)));
            EntityReview {
                key: format!("{entity_type}/{entity_id}"),
                entity_type,
                entity_id,
                kind,
                kind_label: kind.label(),
                kind_tone: kind.tone(),
                advice: kind.advice(),
                not_live: kind == ReviewKind::NewInCode,
                applies: acc.applies,
                fingerprint: fingerprint(kind, &lines),
                lines,
            }
        })
        .collect();
    out.sort_by(|a, b| (a.kind, &a.key).cmp(&(b.kind, &b.key)));
    out
}
