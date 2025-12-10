use crate::models::{CampaignLink, DestinationType, LinkType, UtmParams};
use crate::repository::LinkRepository;
use anyhow::Result;
use chrono::{DateTime, Utc};
use systemprompt_core_database::DbPool;

#[derive(Debug)]
pub struct LinkGenerationService {
    link_repo: LinkRepository,
}

impl LinkGenerationService {
    pub fn new(db: DbPool) -> Self {
        Self {
            link_repo: LinkRepository::new(db),
        }
    }

    pub async fn generate_link(
        &self,
        target_url: &str,
        link_type: LinkType,
        campaign_id: Option<String>,
        campaign_name: Option<String>,
        source_content_id: Option<String>,
        source_page: Option<String>,
        utm_params: Option<UtmParams>,
        link_text: Option<String>,
        link_position: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<CampaignLink> {
        let short_code = Self::generate_short_code();
        let destination_type = Self::determine_destination_type(target_url);

        let utm_json = utm_params.as_ref().map(|p| p.to_json());

        let link = self
            .link_repo
            .create_link(
                &short_code,
                target_url,
                link_type.as_str(),
                source_content_id.as_deref(),
                source_page.as_deref(),
                campaign_id.as_deref(),
                campaign_name.as_deref(),
                utm_json.as_deref(),
                link_text.as_deref(),
                link_position.as_deref(),
                Some(destination_type.as_str()),
                true,
                expires_at,
            )
            .await?;

        Ok(link)
    }

    pub async fn generate_social_media_link(
        &self,
        target_url: &str,
        platform: &str,
        campaign_name: &str,
        source_content_id: Option<String>,
    ) -> Result<CampaignLink> {
        let campaign_id = format!("social_{}_{}", platform, Utc::now().timestamp());

        let utm_params = UtmParams {
            source: Some(platform.to_string()),
            medium: Some("social".to_string()),
            campaign: Some(campaign_name.to_string()),
            term: None,
            content: source_content_id.clone(),
        };

        self.generate_link(
            target_url,
            LinkType::Both,
            Some(campaign_id),
            Some(campaign_name.to_string()),
            source_content_id,
            None,
            Some(utm_params),
            None,
            None,
            None,
        )
        .await
    }

    pub async fn generate_internal_content_link(
        &self,
        target_url: &str,
        source_content_id: &str,
        source_page: &str,
        link_text: Option<String>,
        link_position: Option<String>,
    ) -> Result<CampaignLink> {
        let campaign_id = format!("internal_navigation_{}", Utc::now().date_naive());

        let utm_params = UtmParams {
            source: Some("internal".to_string()),
            medium: Some("content".to_string()),
            campaign: None,
            term: None,
            content: Some(source_content_id.to_string()),
        };

        self.generate_link(
            target_url,
            LinkType::Utm,
            Some(campaign_id),
            Some("Internal Content Navigation".to_string()),
            Some(source_content_id.to_string()),
            Some(source_page.to_string()),
            Some(utm_params),
            link_text,
            link_position,
            None,
        )
        .await
    }

    pub async fn generate_external_cta_link(
        &self,
        target_url: &str,
        campaign_name: &str,
        source_content_id: Option<String>,
        link_text: Option<String>,
    ) -> Result<CampaignLink> {
        let campaign_id = format!("external_cta_{}", Utc::now().timestamp());

        let utm_params = UtmParams {
            source: Some("blog".to_string()),
            medium: Some("cta".to_string()),
            campaign: Some(campaign_name.to_string()),
            term: None,
            content: source_content_id.clone(),
        };

        self.generate_link(
            target_url,
            LinkType::Both,
            Some(campaign_id),
            Some(campaign_name.to_string()),
            source_content_id,
            None,
            Some(utm_params),
            link_text,
            Some("cta".to_string()),
            None,
        )
        .await
    }

    pub async fn generate_external_content_link(
        &self,
        target_url: &str,
        source_content_id: &str,
        source_page: &str,
        link_text: Option<String>,
        link_position: Option<String>,
    ) -> Result<CampaignLink> {
        let campaign_id = format!("social_share_{}", Utc::now().date_naive());

        self.generate_link(
            target_url,
            LinkType::Redirect,
            Some(campaign_id),
            Some("Social Share".to_string()),
            Some(source_content_id.to_string()),
            Some(source_page.to_string()),
            None,
            link_text,
            link_position,
            None,
        )
        .await
    }

    pub async fn get_link_by_short_code(&self, short_code: &str) -> Result<Option<CampaignLink>> {
        Ok(self.link_repo.get_link_by_short_code(short_code).await?)
    }

    pub fn build_trackable_url(link: &CampaignLink, base_url: &str) -> String {
        match link.link_type.as_str() {
            "redirect" | "both" => {
                format!("{}/r/{}", base_url, link.short_code)
            },
            _ => link.target_url.clone(),
        }
    }

    pub fn inject_utm_params(url: &str, utm_params: &UtmParams) -> String {
        let query_string = utm_params.to_query_string();
        if query_string.is_empty() {
            url.to_string()
        } else {
            let separator = if url.contains('?') { "&" } else { "?" };
            format!("{url}{separator}{query_string}")
        }
    }

    fn generate_short_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const CODE_LENGTH: usize = 8;

        let mut rng = rand::thread_rng();
        (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn determine_destination_type(url: &str) -> DestinationType {
        if url.starts_with('/')
            || url.starts_with("http://localhost")
            || url.starts_with("https://localhost")
            || url.contains("tyingshoelaces.com")
            || url.contains("systemprompt.io")
        {
            DestinationType::Internal
        } else {
            DestinationType::External
        }
    }
}
