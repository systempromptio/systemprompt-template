mod rate_limiter;

use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::error::MoltbookError;
use crate::models::*;
use rate_limiter::RateLimiter;

const MOLTBOOK_BASE_URL: &str = "https://www.moltbook.com/api/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct MoltbookClient {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl MoltbookClient {
    pub fn new(api_key: String) -> Result<Self, MoltbookError> {
        Self::with_base_url(api_key, MOLTBOOK_BASE_URL.to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, MoltbookError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent("SystemPrompt-Moltbook/1.0")
            .build()
            .map_err(MoltbookError::Http)?;

        Ok(Self {
            client,
            api_key,
            base_url,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
        })
    }

    async fn check_rate_limit(&self, operation: &str) -> Result<(), MoltbookError> {
        let mut limiter = self.rate_limiter.write().await;
        limiter.check_and_update(operation)
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T, MoltbookError> {
        let status = response.status();

        if status.is_success() {
            response.json::<T>().await.map_err(MoltbookError::Http)
        } else if status.as_u16() == 429 {
            Err(MoltbookError::RateLimitExceeded(
                "Moltbook API rate limit exceeded".to_string(),
            ))
        } else if status.as_u16() == 401 {
            Err(MoltbookError::Unauthorized(
                "Invalid or expired API key".to_string(),
            ))
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(MoltbookError::ApiError {
                status: status.as_u16(),
                message,
            })
        }
    }

    pub async fn create_post(
        &self,
        request: CreatePostRequest,
    ) -> Result<CreatePostResponse, MoltbookError> {
        self.check_rate_limit("post").await?;

        let response = self
            .client
            .post(format!("{}/posts", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn get_post(&self, post_id: &str) -> Result<MoltbookPost, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/posts/{}", self.base_url, post_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn list_posts(
        &self,
        query: ListPostsQuery,
    ) -> Result<Vec<MoltbookPost>, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/posts", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&query)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn create_comment(
        &self,
        request: CreateCommentRequest,
    ) -> Result<CreateCommentResponse, MoltbookError> {
        self.check_rate_limit("comment").await?;

        let response = self
            .client
            .post(format!(
                "{}/posts/{}/comments",
                self.base_url, request.post_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn list_comments(
        &self,
        post_id: &str,
        query: ListCommentsQuery,
    ) -> Result<Vec<MoltbookComment>, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/posts/{}/comments", self.base_url, post_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&query)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn vote_post(
        &self,
        post_id: &str,
        direction: VoteDirection,
    ) -> Result<(), MoltbookError> {
        self.check_rate_limit("vote").await?;

        let response = self
            .client
            .post(format!("{}/posts/{}/vote", self.base_url, post_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&VoteRequest { direction })
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(MoltbookError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Vote failed".to_string()),
            })
        }
    }

    pub async fn vote_comment(
        &self,
        comment_id: &str,
        direction: VoteDirection,
    ) -> Result<(), MoltbookError> {
        self.check_rate_limit("vote").await?;

        let response = self
            .client
            .post(format!("{}/comments/{}/vote", self.base_url, comment_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&VoteRequest { direction })
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(MoltbookError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Vote failed".to_string()),
            })
        }
    }

    pub async fn search_posts(
        &self,
        query: PostSearchQuery,
    ) -> Result<Vec<MoltbookPost>, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/search/posts", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&query)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn get_submolt(&self, name: &str) -> Result<Submolt, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/submolts/{}", self.base_url, name))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn search_submolts(
        &self,
        query: SubmoltSearchQuery,
    ) -> Result<Vec<Submolt>, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/search/submolts", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&query)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn get_feed(&self, limit: Option<i64>) -> Result<Vec<MoltbookPost>, MoltbookError> {
        self.check_rate_limit("read").await?;

        let query = ListPostsQuery {
            limit,
            ..Default::default()
        };

        let response = self
            .client
            .get(format!("{}/feed", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&query)
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }

    pub async fn get_agent_profile(&self, agent_id: &str) -> Result<AgentProfile, MoltbookError> {
        self.check_rate_limit("read").await?;

        let response = self
            .client
            .get(format!("{}/agents/{}", self.base_url, agent_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(MoltbookError::Http)?;

        self.handle_response(response).await
    }
}
