//! Applying an access template: one subject written for, or cleared from,
//! every entity of a kind, with the outcome tallied per entity.

use systemprompt_security::authz::{
    Access, AccessControlRepository, DASHBOARD_SOURCE, EntityKind, RuleType, UpsertRuleParams,
};

// Why: the subject a template writes for, the same for every entity it
// touches.
pub(super) struct TemplateSubject<'a> {
    pub(super) kind: EntityKind,
    pub(super) rule_type: &'a RuleType,
    pub(super) rule_value: &'a str,
    pub(super) justification: Option<&'a str>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Tally {
    pub(super) applied: usize,
    pub(super) failed: usize,
}

impl std::ops::AddAssign for Tally {
    fn add_assign(&mut self, rhs: Self) {
        self.applied += rhs.applied;
        self.failed += rhs.failed;
    }
}

pub(super) async fn clear_subject_rules(
    r: &AccessControlRepository,
    subject: &TemplateSubject<'_>,
    entity_id: &str,
) -> Tally {
    let existing = r
        .list_rules_for_entity(subject.kind, entity_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity access: rule listing failed"))
        .unwrap_or_default();
    let mut tally = Tally::default();
    for rule in existing {
        if &rule.rule_type == subject.rule_type && rule.rule_value == subject.rule_value {
            if r.delete_rule(&rule.id).await.is_ok() {
                tally.applied += 1;
            } else {
                tally.failed += 1;
            }
        }
    }
    tally
}

pub(super) async fn upsert_subject_rule(
    r: &AccessControlRepository,
    subject: &TemplateSubject<'_>,
    entity_id: &str,
    access: Access,
) -> Tally {
    match r
        .upsert_rule(UpsertRuleParams {
            entity_type: subject.kind,
            entity_id,
            rule_type: subject.rule_type.clone(),
            rule_value: subject.rule_value,
            access,
            justification: subject.justification,
            source: DASHBOARD_SOURCE,
        })
        .await
    {
        Ok(_) => Tally {
            applied: 1,
            failed: 0,
        },
        Err(e) => {
            tracing::warn!(error = %e, entity_id = %entity_id, "apply_template upsert failed");
            Tally {
                applied: 0,
                failed: 1,
            }
        },
    }
}
