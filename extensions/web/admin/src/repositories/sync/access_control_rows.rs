//! The access-control drift as rows of the generic diff table.
//!
//! One row per disagreement, in a fixed vocabulary of `kind` tokens the page
//! and the tests key on (`missing`, `orphan`, `changed`, `default`, `entity`,
//! `awaiting`). The label says what happened in words a reader acts on —
//! *removed from code*, *written in console* — and the resolution badge names
//! the one button that settles it. Orphans stamped `bundle:<name>` — rows a
//! kit's own `access:` block wrote — are labelled as such so the operator
//! sees which kit is carrying a second truth.

use super::plane::{DriftRow, resolution};
use crate::repositories::access_control::declared::{DeclaredEntity, DeclaredRule, DeclaredSet};
use crate::repositories::access_control::drift::{
    ChangedRule, DefaultDrift, DriftReport, OrphanOrigin, OrphanRule,
};

fn label(entity_type: &str) -> String {
    entity_type.replace('_', " ")
}

#[must_use]
pub fn band_label(rule_type: &str) -> &'static str {
    match rule_type {
        "user" => "person",
        "project" => "project",
        "group" => "group",
        "connector" => "connected server",
        "role" => "role",
        "organization" => "organization",
        _ => "band",
    }
}

pub(super) fn awaiting_rows(declared: &DeclaredSet) -> Vec<DriftRow> {
    declared
        .awaiting
        .iter()
        .map(|a| DriftRow {
            kind: "awaiting",
            kind_label: "Awaiting its bundle",
            kind_tone: "muted",
            entity_type_label: label(&a.entity_type),
            entity_type: a.entity_type.clone(),
            entity_id: a.entity_id.clone(),
            band: String::new(),
            band_label: "owner",
            subject: a.owner.clone(),
            in_code: "declared; validated once the bundle is active".to_owned(),
            in_db: "—".to_owned(),
            origin: "",
            origin_tone: "muted",
            governed: false,
            insert_applies: false,
            overwrite_applies: false,
            overwrite_effect: "no change until the bundle is pinned",
            resolve: "Pin the bundle".to_owned(),
            resolve_tone: "muted",
        })
        .collect()
}

// Why: label, tone and provenance badge for a database-only row, by who
// wrote it. The label is the cause in plain words; `stale` never appears.
const fn orphan_words(
    orphan: &OrphanRule,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match orphan.origin {
        OrphanOrigin::Code => ("Removed from code", "err", "code", "muted"),
        OrphanOrigin::Console => ("Written in console", "info", "console", "info"),
        OrphanOrigin::Bundle => ("Owned by a bundle", "muted", "bundle", "muted"),
    }
}

#[must_use]
pub fn rows(drift: &DriftReport) -> Vec<DriftRow> {
    let mut out = Vec::new();
    out.extend(drift.missing_in_db.iter().map(missing_row));
    out.extend(drift.only_in_db.iter().map(orphan_row));
    out.extend(drift.changed.iter().map(changed_row));
    out.extend(drift.default_changed.iter().map(default_row));
    out.extend(drift.entities_missing.iter().map(entity_row));
    out.sort_by(|a, b| {
        (&a.entity_type, &a.entity_id, &a.band, &a.subject).cmp(&(
            &b.entity_type,
            &b.entity_id,
            &b.band,
            &b.subject,
        ))
    });
    out
}

fn missing_row(rule: &DeclaredRule) -> DriftRow {
    let (resolve, resolve_tone) = resolution(true, true, "inserted");
    DriftRow {
        kind: "missing",
        kind_label: "Added in code",
        kind_tone: "ok",
        entity_type_label: label(&rule.key.entity_type),
        entity_type: rule.key.entity_type.clone(),
        entity_id: rule.key.entity_id.clone(),
        band_label: band_label(&rule.key.rule_type),
        band: rule.key.rule_type.clone(),
        subject: rule.key.rule_value.clone(),
        in_code: format!("{} — {}", rule.access, rule.justification),
        in_db: "—".to_owned(),
        origin: "",
        origin_tone: "muted",
        governed: true,
        insert_applies: true,
        overwrite_applies: true,
        overwrite_effect: "inserted",
        resolve,
        resolve_tone,
    }
}

fn orphan_row(orphan: &OrphanRule) -> DriftRow {
    let (kind_label, kind_tone, origin, origin_tone) = orphan_words(orphan);
    let (resolve, resolve_tone) = match (orphan.retire, orphan.origin) {
        (true, _) => ("Overwrite deletes".to_owned(), "err"),
        (false, OrphanOrigin::Console) => ("Export to keep".to_owned(), "info"),
        (false, _) => ("Kept — the bundle's own truth".to_owned(), "muted"),
    };
    DriftRow {
        kind: "orphan",
        kind_label,
        kind_tone,
        entity_type_label: label(&orphan.row.entity_type),
        entity_type: orphan.row.entity_type.clone(),
        entity_id: orphan.row.entity_id.clone(),
        band_label: band_label(&orphan.row.rule_type),
        band: orphan.row.rule_type.clone(),
        subject: orphan.row.rule_value.clone(),
        in_code: "—".to_owned(),
        in_db: format!(
            "{} — {}",
            orphan.row.access,
            orphan
                .row
                .justification
                .as_deref()
                .unwrap_or("(no reason recorded)")
        ),
        origin,
        origin_tone,
        governed: orphan.governed,
        insert_applies: false,
        overwrite_applies: orphan.retire,
        overwrite_effect: if orphan.retire {
            "deleted"
        } else {
            "kept (entity not in code)"
        },
        resolve,
        resolve_tone,
    }
}

fn changed_row(changed: &ChangedRule) -> DriftRow {
    let console = changed.db_source == "dashboard";
    let (resolve, resolve_tone) = resolution(false, true, "updated");
    DriftRow {
        kind: "changed",
        kind_label: if console {
            "Edited in console"
        } else {
            "Changed in code"
        },
        kind_tone: "warn",
        entity_type_label: label(&changed.key.entity_type),
        entity_type: changed.key.entity_type.clone(),
        entity_id: changed.key.entity_id.clone(),
        band_label: band_label(&changed.key.rule_type),
        band: changed.key.rule_type.clone(),
        subject: changed.key.rule_value.clone(),
        in_code: format!("{} — {}", changed.declared_access, changed.declared_why),
        in_db: format!(
            "{} — {}",
            changed.db_access,
            changed.db_why.as_deref().unwrap_or("(no reason recorded)")
        ),
        origin: if console { "console" } else { "code" },
        origin_tone: if console { "info" } else { "muted" },
        governed: true,
        insert_applies: false,
        overwrite_applies: true,
        overwrite_effect: "updated",
        resolve,
        resolve_tone,
    }
}

fn default_row(d: &DefaultDrift) -> DriftRow {
    let (resolve, resolve_tone) = resolution(false, true, "updated");
    DriftRow {
        kind: "default",
        kind_label: "Default changed in code",
        kind_tone: "warn",
        entity_type_label: label(&d.entity_type),
        entity_type: d.entity_type.clone(),
        entity_id: d.entity_id.clone(),
        band: String::new(),
        band_label: "default",
        subject: "(entity default)".to_owned(),
        in_code: if d.declared_open { "open" } else { "closed" }.to_owned(),
        in_db: if d.db_open { "open" } else { "closed" }.to_owned(),
        origin: "",
        origin_tone: "muted",
        governed: true,
        insert_applies: false,
        overwrite_applies: true,
        overwrite_effect: "updated",
        resolve,
        resolve_tone,
    }
}

fn entity_row(e: &DeclaredEntity) -> DriftRow {
    let (resolve, resolve_tone) = resolution(true, true, "inserted");
    DriftRow {
        kind: "entity",
        kind_label: "Entity added in code",
        kind_tone: "ok",
        entity_type_label: label(&e.entity_type),
        entity_type: e.entity_type.clone(),
        entity_id: e.entity_id.clone(),
        band: String::new(),
        band_label: "default",
        subject: "(entity default)".to_owned(),
        in_code: if e.default_included { "open" } else { "closed" }.to_owned(),
        in_db: "—".to_owned(),
        origin: "",
        origin_tone: "muted",
        governed: true,
        insert_applies: true,
        overwrite_applies: true,
        overwrite_effect: "inserted",
        resolve,
        resolve_tone,
    }
}
