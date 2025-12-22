//! Link generation service for creating trackable links.

use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CampaignId, ContentId};

use crate::error::BlogError;
use crate::models::{CampaignLink, CreateLinkParams, UtmParams};
use crate::repository::LinkRepository;

/// Service for generating campaign links.
#[derive(Debug, Clone)]
pub struct LinkGenerationService {
    repo: LinkRepository,
}

impl LinkGenerationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: LinkRepository::new(pool),
        }
    }

    /// Generate a new tracking link.
    pub async fn generate(
        &self,
        target_url: String,
        campaign_name: Option<String>,
        utm_params: Option<UtmParams>,
    ) -> Result<CampaignLink, BlogError> {
        let utm_json = match utm_params {
            Some(ref params) => Some(params.to_json()?),
            None => None,
        };

        let short_code = generate_short_code();
        let mut params = CreateLinkParams::new(short_code, target_url, "redirect".to_string());

        if let Some(name) = campaign_name {
            let campaign_id = CampaignId::new(uuid::Uuid::new_v4().to_string());
            params = params
                .with_campaign_id(Some(campaign_id))
                .with_campaign_name(Some(name));
        }

        params = params.with_utm_params(utm_json);

        self.repo.create_link(&params).await.map_err(BlogError::from)
    }

    /// Generate a link for content.
    pub async fn generate_for_content(
        &self,
        target_url: String,
        content_id: String,
        campaign_name: Option<String>,
    ) -> Result<CampaignLink, BlogError> {
        let short_code = generate_short_code();
        let content_id = ContentId::new(content_id);
        let mut params = CreateLinkParams::new(short_code, target_url, "redirect".to_string())
            .with_source_content_id(Some(content_id));

        if let Some(name) = campaign_name {
            let campaign_id = CampaignId::new(uuid::Uuid::new_v4().to_string());
            params = params
                .with_campaign_id(Some(campaign_id))
                .with_campaign_name(Some(name));
        }

        self.repo.create_link(&params).await.map_err(BlogError::from)
    }
}

fn generate_short_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
