//! The Access review tab of `/admin/sync`: the access-control drift as one
//! row per entity, each settled on its own — apply code, keep the database,
//! or export — with the whole-plane card below for the rare wholesale move.

use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::handlers::ssr::entity_panel::entity_access_url;
use crate::handlers::ssr::sync_plane::{PlaneCardView, plane_card_by_id};
use crate::repositories::sync::access_control::{PLANE_ID, declared_now, drift_now};
use crate::repositories::sync::attention::{ReviewSplit, ReviewedEntity, reviews_for};

#[derive(Debug, Serialize)]
pub(crate) struct ReviewRowView {
    #[serde(flatten)]
    pub entity: ReviewedEntity,
    pub entity_url: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AccessReviewView {
    pub pending: Vec<ReviewRowView>,
    pub kept: Vec<ReviewRowView>,
    pub pending_count: usize,
    pub kept_count: usize,
    pub not_live_count: usize,
    pub unreadable: Option<String>,
    pub export_url: String,
    pub plane: Option<PlaneCardView>,
}

fn rows(list: Vec<ReviewedEntity>) -> Vec<ReviewRowView> {
    list.into_iter()
        .map(|entity| ReviewRowView {
            entity_url: entity_access_url(&entity.review.entity_type, &entity.review.entity_id),
            entity,
        })
        .collect()
}

async fn split(pool: &PgPool) -> Result<ReviewSplit, String> {
    let declared = declared_now().await.map_err(|e| e.to_string())?;
    let drift = drift_now(pool, &declared)
        .await
        .map_err(|e| e.to_string())?;
    reviews_for(pool, &drift).await.map_err(|e| e.to_string())
}

pub(super) async fn access_review(pool: &PgPool) -> AdminResult<AccessReviewView> {
    let mut view = AccessReviewView {
        export_url: format!("/api/public/admin/sync/planes/{PLANE_ID}/export"),
        plane: plane_card_by_id(pool, PLANE_ID, "").await?,
        ..AccessReviewView::default()
    };
    match split(pool).await {
        Ok(split) => {
            view.not_live_count = split.pending.iter().filter(|r| r.review.not_live).count();
            view.pending_count = split.pending.len();
            view.pending = rows(split.pending);
            view.kept_count = split.kept.len();
            view.kept = rows(split.kept);
        },
        Err(e) => view.unreadable = Some(e),
    }
    Ok(view)
}
