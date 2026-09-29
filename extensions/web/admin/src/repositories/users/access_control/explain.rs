//! "Why does this person (not) reach this entity?" — every band the resolver
//! considers, what each would decide alone, and which one actually decided.
//!
//! The resolver returns one decision and the band that produced it; it does
//! not report the bands it passed over. This module asks it once per band,
//! each time with a subject that holds only that band's values and a closed
//! default, so a band's own verdict is the resolver's answer and never a
//! re-implementation of its ladder. The real decision is then asked with the
//! whole subject, and each band is marked *decided*, *outranked* (it would
//! have decided something but a narrower band won) or *no rule*.

use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    ParentChainIndex, RuleType, SubjectAttributes, SubjectDimension,
};

use super::matrix_resolution::{MatrixCell, resolve_effective_with};
use super::matrix_subject::MatrixSubject;
use crate::types::access_control::AccessControlRule;

// Why: never a real account id — `explain:` is not a legal prefix for an
// email or a directory id — so a user-band rule cannot fire for a band
// evaluated in isolation.
const PLACEHOLDER_ID: &str = "explain:band";

#[derive(Debug, Clone, Serialize)]
pub struct BandExplanation {
    pub band: String,
    pub label: String,
    pub precedence: u16,
    pub held: Vec<String>,
    pub outcome: &'static str,
    pub detail: String,
    pub verdict: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Explanation {
    pub effective: String,
    pub layer: String,
    pub detail: String,
    pub default_open: bool,
    pub bands: Vec<BandExplanation>,
}

/// Everything one explanation reads: the rule table, the entity, the whole
/// subject, the registered dimensions and the parent chains.
#[derive(Debug, Clone, Copy)]
pub struct ExplainInput<'a> {
    pub rules: &'a [AccessControlRule],
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub subject: &'a MatrixSubject,
    pub dimensions: &'a [SubjectDimension],
    pub default_open: bool,
    pub chains: &'a ParentChainIndex,
}

struct Band {
    rule_type: RuleType,
    label: String,
    precedence: u16,
}

fn bands(dimensions: &[SubjectDimension]) -> Vec<Band> {
    let mut out = vec![Band {
        rule_type: RuleType::USER,
        label: "person".to_owned(),
        precedence: 0,
    }];
    for d in dimensions {
        if out.iter().any(|b| b.rule_type == d.rule_type) {
            continue;
        }
        out.push(Band {
            rule_type: d.rule_type.clone(),
            label: d.label.to_owned(),
            precedence: d.precedence,
        });
    }
    if !out.iter().any(|b| b.rule_type == RuleType::ROLE) {
        out.push(Band {
            rule_type: RuleType::ROLE,
            label: "role".to_owned(),
            precedence: 200,
        });
    }
    out.sort_by_key(|b| b.precedence);
    out
}

fn held(band: &Band, subject: &MatrixSubject) -> Vec<String> {
    if band.rule_type == RuleType::USER {
        vec![subject.id.as_str().to_owned()]
    } else if band.rule_type == RuleType::ROLE {
        subject.roles.clone()
    } else {
        subject.attributes.values(&band.rule_type).to_vec()
    }
}

fn isolated(band: &Band, subject: &MatrixSubject, values: Vec<String>) -> MatrixSubject {
    let mut attributes = SubjectAttributes::new();
    let (id, roles) = if band.rule_type == RuleType::USER {
        (subject.id.clone(), Vec::new())
    } else if band.rule_type == RuleType::ROLE {
        (UserId::new(PLACEHOLDER_ID), values)
    } else {
        attributes.insert(band.rule_type.clone(), values);
        (UserId::new(PLACEHOLDER_ID), Vec::new())
    };
    MatrixSubject {
        id,
        roles,
        attributes,
    }
}

fn resolve(
    input: &ExplainInput<'_>,
    subject: &MatrixSubject,
    open: bool,
) -> (String, String, String) {
    let (effective, source) = resolve_effective_with(
        &MatrixCell {
            all_rules: input.rules,
            entity_type: input.entity_type,
            entity_id: input.entity_id,
            subject_id: &subject.id,
            subject_roles: &subject.roles,
            attributes: &subject.attributes,
            dimensions: input.dimensions,
            default_included: open,
        },
        input.chains,
    );
    (effective, source.layer, source.detail)
}

#[must_use]
pub fn explain(input: &ExplainInput<'_>) -> Explanation {
    let (effective, layer, detail) = resolve(input, input.subject, input.default_open);
    let bands = bands(input.dimensions)
        .into_iter()
        .map(|band| {
            let values = held(&band, input.subject);
            let (outcome, band_detail) = if values.is_empty() {
                ("none", String::new())
            } else {
                let alone = isolated(&band, input.subject, values.clone());
                let (eff, band_layer, d) = resolve(input, &alone, false);
                match (eff.as_str(), band_layer.as_str()) {
                    (_, "default") => ("none", String::new()),
                    ("allow", _) => ("allow", d),
                    ("deny", _) => ("deny", d),
                    _ => ("none", d),
                }
            };
            let verdict = match outcome {
                _ if values.is_empty() => "not_held",
                "none" => "no_rule",
                _ if band.rule_type.as_str() == layer => "decided",
                _ => "outranked",
            };
            BandExplanation {
                band: band.rule_type.as_str().to_owned(),
                label: band.label,
                precedence: band.precedence,
                held: values,
                outcome,
                detail: band_detail,
                verdict,
            }
        })
        .collect();
    Explanation {
        effective,
        layer,
        detail,
        default_open: input.default_open,
        bands,
    }
}

// Why: `None` when the account does not exist — the panel says so rather
// than explaining a subject nobody holds.
pub async fn explain_for_user(
    pool: &sqlx::PgPool,
    user_id: &UserId,
    entity_type: &str,
    entity_id: &str,
) -> Result<Option<Explanation>, sqlx::Error> {
    let Some(user) = super::matrix::find_user_for_matrix(pool, user_id).await? else {
        return Ok(None);
    };
    let subject = super::matrix_subject::user_subject(pool, user_id, user.roles).await?;
    let inputs = super::matrix::resolution_inputs(pool).await?;
    let default_open = inputs
        .defaults
        .get(&(entity_type.to_owned(), entity_id.to_owned()))
        .copied()
        .unwrap_or(false);
    Ok(Some(explain(&ExplainInput {
        rules: &inputs.rules,
        entity_type,
        entity_id,
        subject: &subject,
        dimensions: crate::authz::dimensions(pool),
        default_open,
        chains: &inputs.chains,
    })))
}
