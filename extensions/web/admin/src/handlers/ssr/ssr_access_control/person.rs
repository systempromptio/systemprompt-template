//! The "Find a person" tab: a search over every account, each hit linking to
//! that person's own Access tab — the one place their access is read and
//! edited. Nothing is capped but the hit list, so anyone can be found.
//!
//! A plain GET form, so the tab works without JavaScript and a search is a
//! link someone can send.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories;

const HIT_LIMIT: usize = 25;

#[derive(Debug, Serialize)]
pub(crate) struct PersonHitView {
    pub name: String,
    pub email: String,
    pub href: String,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct PersonCheckView {
    pub query: String,
    pub searched: bool,
    pub hits: Vec<PersonHitView>,
    pub more: bool,
}

pub(super) async fn build(pool: &PgPool, query: Option<&str>) -> PersonCheckView {
    let Some(query) = query.map(str::trim).filter(|q| !q.is_empty()) else {
        return PersonCheckView::default();
    };
    let wanted = query.to_lowercase();
    let matches: Vec<PersonHitView> =
        repositories::users::queries::list_users(pool, &repositories::scope::SubjectScope::All)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "find a person: user listing failed"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|u| {
                let id = u.user_id.as_str().to_owned();
                let email = u
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let name = u.display_name.unwrap_or_default();
                let hit = [&id, &email, &name]
                    .iter()
                    .any(|field| field.to_lowercase().contains(&wanted));
                hit.then(|| PersonHitView {
                    href: format!("/admin/users/{}?tab=access", urlencoding::encode(&id)),
                    name: if name.trim().is_empty() { id } else { name },
                    email,
                })
            })
            .take(HIT_LIMIT + 1)
            .collect();
    let more = matches.len() > HIT_LIMIT;
    PersonCheckView {
        query: query.to_owned(),
        searched: true,
        hits: matches.into_iter().take(HIT_LIMIT).collect(),
        more,
    }
}
