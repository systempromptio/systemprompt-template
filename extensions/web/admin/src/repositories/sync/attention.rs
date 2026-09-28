//! The access-control review split into what still waits on a person and
//! what somebody already decided to keep, and the count the sidebar shows.
//!
//! The count is process-local and refreshed whenever the plane's drift is
//! read from disk — at boot, on every access-control or sync page, and after
//! every apply — so the shell can print it without a query per page. It can
//! lag a console edit made on another node until that node reads the drift.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;
use sqlx::PgPool;

use super::history::{KeptReview, list_kept_reviews};
use crate::repositories::access_control::drift::DriftReport;
use crate::repositories::access_control::review::{EntityReview, review_entities};

static ACCESS_ATTENTION: AtomicUsize = AtomicUsize::new(0);

// Why: lint-ok: unused-pub — read by the console shell for its sidebar count, which lands with the
// Stage-3 admin port.
#[must_use]
pub fn access_attention() -> usize {
    ACCESS_ATTENTION.load(Ordering::Relaxed)
}

pub fn record_access_attention(count: usize) {
    ACCESS_ATTENTION.store(count, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize)]
pub struct KeptView {
    pub by: String,
    pub reason: String,
    pub at: String,
}

impl From<&KeptReview> for KeptView {
    fn from(k: &KeptReview) -> Self {
        Self {
            by: k.display_name.clone(),
            reason: k.reason.clone(),
            at: k.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewedEntity {
    #[serde(flatten)]
    pub review: EntityReview,
    pub kept: Option<KeptView>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ReviewSplit {
    pub pending: Vec<ReviewedEntity>,
    pub kept: Vec<ReviewedEntity>,
}

// Why: a decision holds only for the diff it was taken against. When either
// side moves the fingerprint changes and the entity is back on the list.
#[must_use]
pub fn split_reviews(reviews: Vec<EntityReview>, kept: &[KeptReview]) -> ReviewSplit {
    let mut split = ReviewSplit::default();
    for review in reviews {
        let decision = kept
            .iter()
            .find(|k| k.key == review.key && k.fingerprint == review.fingerprint);
        match decision {
            Some(k) => split.kept.push(ReviewedEntity {
                review,
                kept: Some(KeptView::from(k)),
            }),
            None => split.pending.push(ReviewedEntity { review, kept: None }),
        }
    }
    split
}

pub async fn reviews_for(pool: &PgPool, drift: &DriftReport) -> Result<ReviewSplit, sqlx::Error> {
    let kept = list_kept_reviews(pool).await?;
    let split = split_reviews(review_entities(drift), &kept);
    record_access_attention(split.pending.len());
    Ok(split)
}
