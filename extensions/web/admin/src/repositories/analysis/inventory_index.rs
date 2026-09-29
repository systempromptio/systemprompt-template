//! Marketplace → plugin → skill reachability and the people each marketplace
//! reaches, assembled from the services tree and the access-control resolver.
//!
//! Membership is authored in YAML (`services/marketplaces`,
//! `services/plugins`), so the index is rebuilt per request rather than
//! persisted. It describes the catalog as placed today; historical placement is
//! not a retained dimension.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::marketplace::manifests::{
    MarketplaceConfigSummary, list_marketplace_configs,
};
use crate::repositories::marketplace::plugin_maps::{EntityPluginMap, build_entity_plugin_maps};
use crate::repositories::users::access_control::{
    MatrixSubject, SectionInput, resolve_subject_matrices, user_subject,
};

const MARKETPLACE_ENTITY: &str = "marketplace";
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct InventoryIndex {
    pub marketplaces: Vec<MarketplaceConfigSummary>,
    pub plugins_by_skill: EntityPluginMap,
    pub marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>>,
}

impl InventoryIndex {
    #[must_use]
    pub fn build(services_path: &Path) -> Self {
        let marketplaces = list_marketplace_configs(services_path).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "marketplace manifests unavailable for analysis");
            Vec::new()
        });
        let (plugins_by_skill, _agents, _mcp) = build_entity_plugin_maps(services_path);
        let mut marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>> = HashMap::new();
        for marketplace in &marketplaces {
            for plugin in &marketplace.plugins {
                marketplaces_by_plugin
                    .entry(plugin.clone())
                    .or_default()
                    .push(MarketplaceRef {
                        id: marketplace.id.as_str().to_owned(),
                        name: marketplace.name.clone(),
                    });
            }
        }
        Self {
            marketplaces,
            plugins_by_skill,
            marketplaces_by_plugin,
        }
    }
}

/// The distinct users each marketplace reaches, resolved per group and per
/// role and then materialised through memberships.
#[derive(Debug, Default)]
pub struct MarketplaceAudience {
    pub users_by_marketplace: HashMap<String, HashSet<UserId>>,
}

impl MarketplaceAudience {
    #[must_use]
    pub fn users_reaching_skill(&self, index: &InventoryIndex, skill: &str) -> usize {
        let mut users: HashSet<&UserId> = HashSet::new();
        for plugin in index.plugins_by_skill.get(skill).into_iter().flatten() {
            for marketplace in index
                .marketplaces_by_plugin
                .get(&plugin.0)
                .into_iter()
                .flatten()
            {
                if let Some(set) = self.users_by_marketplace.get(&marketplace.id) {
                    users.extend(set.iter());
                }
            }
        }
        users.len()
    }

    #[must_use]
    pub fn users_reaching_marketplace(&self, marketplace: &str) -> usize {
        self.users_by_marketplace
            .get(marketplace)
            .map_or(0, HashSet::len)
    }

    // Why: installs are receipts from anyone who ever installed; a rate over
    // the entitled must count only the installed who are entitled, or one
    // extra consumer reads as 200%.
    #[must_use]
    pub fn entitled_among(&self, marketplace: &str, consumers: &[String]) -> usize {
        self.users_by_marketplace.get(marketplace).map_or(0, |set| {
            let ids: HashSet<&str> = set.iter().map(UserId::as_str).collect();
            consumers
                .iter()
                .filter(|c| ids.contains(c.as_str()))
                .count()
        })
    }

    #[must_use]
    pub fn entitled_among_any(&self, consumers: &[String]) -> usize {
        let ids: HashSet<&str> = self
            .users_by_marketplace
            .values()
            .flatten()
            .map(UserId::as_str)
            .collect();
        consumers
            .iter()
            .filter(|c| ids.contains(c.as_str()))
            .count()
    }

    #[must_use]
    pub fn users_reaching_any_marketplace(&self) -> usize {
        self.users_by_marketplace
            .values()
            .flatten()
            .collect::<HashSet<_>>()
            .len()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MembershipRow {
    user_id: UserId,
    roles: Vec<String>,
}

// Why: bounded so a large roster does not open one attribute lookup per
// user at once against the pool; the audience is read per page render.
const AUDIENCE_CONCURRENCY: usize = 8;

// Why: audience is every signed-in person resolved as the subject the
// enforcement point sees — own roles plus every registered dimension
// (groups, projects, connectors, linked Salesforce identities) — so a count
// here and a decision at the gateway cannot disagree. A marketplace opened by
// a connected server reaches exactly the people holding that connection;
// group and role bands alone would count it as reaching nobody.
pub async fn get_marketplace_audience(
    pool: &PgPool,
    manifests: &[MarketplaceConfigSummary],
) -> Result<MarketplaceAudience, sqlx::Error> {
    let members = sqlx::query_as::<_, MembershipRow>(
        "SELECT id AS user_id, roles FROM users WHERE NOT ('anonymous' = ANY(roles)) ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let subjects = gather_subjects(pool, members).await?;
    let sections: Vec<SectionInput> = vec![(
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        manifests
            .iter()
            .map(|m| (m.id.as_str().to_owned(), m.name.clone(), None))
            .collect(),
    )];
    let resolved = resolve_subject_matrices(pool, &subjects, &sections).await?;
    let mut users_by_marketplace: HashMap<String, HashSet<UserId>> = manifests
        .iter()
        .map(|m| (m.id.as_str().to_owned(), HashSet::new()))
        .collect();
    for (subject, sections) in subjects.iter().zip(resolved) {
        for row in sections.into_iter().flat_map(|s| s.rows) {
            if row.effective == "allow" {
                users_by_marketplace
                    .entry(row.entity_id)
                    .or_default()
                    .insert(subject.id.clone());
            }
        }
    }
    Ok(MarketplaceAudience {
        users_by_marketplace,
    })
}

async fn gather_subjects(
    pool: &PgPool,
    members: Vec<MembershipRow>,
) -> Result<Vec<MatrixSubject>, sqlx::Error> {
    let mut subjects = Vec::with_capacity(members.len());
    for chunk in members.chunks(AUDIENCE_CONCURRENCY) {
        let mut set = tokio::task::JoinSet::new();
        for member in chunk {
            let pool = pool.clone();
            let user_id = member.user_id.clone();
            let roles = member.roles.clone();
            set.spawn(async move { user_subject(&pool, &user_id, roles).await });
        }
        while let Some(joined) = set.join_next().await {
            let subject = joined.map_err(|e| sqlx::Error::Protocol(e.to_string()))??;
            subjects.push(subject);
        }
    }
    subjects.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(subjects)
}
