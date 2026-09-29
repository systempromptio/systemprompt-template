//! The panel's "Why?" explainer: one person against this entity, every band
//! of the ladder with what it would decide alone and which one did.
//!
//! The person is typed, not picked from a capped list: an account id or an
//! email, matched against every account. The query string is the only
//! input, so the explainer is a plain GET form and a link someone can send.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::view::{WhyBandView, WhyView};
use crate::repositories;
use crate::repositories::users::access_control::explain::{Explanation, explain_for_user};

async fn find_person(pool: &PgPool, query: &str) -> Option<(UserId, String)> {
    let wanted = query.trim().to_lowercase();
    repositories::users::queries::list_users(pool, &repositories::scope::SubjectScope::All)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: user listing failed"))
        .unwrap_or_default()
        .into_iter()
        .find(|u| {
            u.user_id.as_str().to_lowercase() == wanted
                || u.email
                    .as_ref()
                    .is_some_and(|e| e.to_string().to_lowercase() == wanted)
        })
        .map(|u| {
            let label = u
                .email
                .as_ref()
                .map(ToString::to_string)
                .or(u.display_name)
                .unwrap_or_else(|| u.user_id.as_str().to_owned());
            (u.user_id, label)
        })
}

const fn verdict_words(verdict: &str) -> (&'static str, &'static str) {
    match verdict.as_bytes() {
        b"decided" => ("Decided", "accent"),
        b"outranked" => ("Outranked", "warn"),
        b"no_rule" => ("No rule", "muted"),
        _ => ("Not held", "muted"),
    }
}

fn bands_of(explanation: &Explanation) -> Vec<WhyBandView> {
    explanation
        .bands
        .iter()
        .map(|b| {
            let (verdict_label, verdict_tone) = verdict_words(b.verdict);
            WhyBandView {
                label: b.label.clone(),
                precedence: b.precedence,
                held: if b.held.is_empty() {
                    "—".to_owned()
                } else {
                    b.held.join(", ")
                },
                decision: b.outcome,
                verdict_label,
                verdict_tone,
                detail: b.detail.clone(),
            }
        })
        .collect()
}

pub(super) async fn why(pool: &PgPool, query: &str, entity: (&str, &str)) -> WhyView {
    let mut view = WhyView {
        query: query.to_owned(),
        found: false,
        person: String::new(),
        decision: String::new(),
        decided_by: String::new(),
        detail: String::new(),
        bands: Vec::new(),
    };
    let Some((user_id, label)) = find_person(pool, query).await else {
        return view;
    };
    let explained = explain_for_user(pool, &user_id, entity.0, entity.1)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: explanation failed"))
        .ok()
        .flatten();
    let Some(explanation) = explained else {
        return view;
    };
    view.found = true;
    view.person = label;
    view.bands = bands_of(&explanation);
    view.decision = explanation.effective;
    view.decided_by = explanation.layer;
    view.detail = explanation.detail;
    view
}
