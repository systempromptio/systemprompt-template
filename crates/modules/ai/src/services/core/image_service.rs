use crate::errors::{AiError, Result};
use crate::models::image_generation::{
    ImageGenerationRequest, ImageGenerationResponse,
};
use crate::repository::ai_requests::AiRequestRepository;
use crate::repository::image_repository::ImageRepository;
use crate::services::providers::image_provider_trait::BoxedImageProvider;
use crate::storage::{ImageStorage, StorageConfig};
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use uuid::Uuid;

pub struct ImageService {
    providers: HashMap<String, BoxedImageProvider>,
    storage: Arc<ImageStorage>,
    image_repo: ImageRepository,
    ai_request_repo: AiRequestRepository,
    default_provider: Option<String>,
}

impl ImageService {
    pub fn new(db_pool: DbPool, storage_config: StorageConfig) -> Result<Self> {
        let storage = Arc::new(ImageStorage::new(storage_config)?);
        let image_repo = ImageRepository::new(db_pool.clone());
        let ai_request_repo = AiRequestRepository::new(db_pool);

        Ok(Self {
            providers: HashMap::new(),
            storage,
            image_repo,
            ai_request_repo,
            default_provider: None,
        })
    }

    pub fn register_provider(&mut self, provider: BoxedImageProvider) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    pub fn set_default_provider(&mut self, provider_name: String) {
        self.default_provider = Some(provider_name);
    }

    pub fn get_provider(&self, name: &str) -> Option<&BoxedImageProvider> {
        self.providers.get(name)
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn generate_image(
        &self,
        mut request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse> {
        // Determine which provider to use
        let provider_name = if let Some(model) = &request.model {
            self.find_provider_for_model(model)?
        } else if let Some(default) = &self.default_provider {
            default.clone()
        } else {
            return Err(AiError::ConfigurationError(
                "No model specified and no default provider configured".to_string(),
            ));
        };

        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| AiError::ProviderError {
                    provider: provider_name.clone(),
                    message: "Provider not found".to_string(),
                })?;

        // Set trace_id if not provided
        if request.trace_id.is_none() {
            request.trace_id = Some(Uuid::new_v4().to_string());
        }

        // Generate the image
        let mut response = provider.generate_image(&request).await?;

        // Set cost estimate
        response.cost_estimate = Some(provider.capabilities().cost_per_image_cents);

        // Save image to storage
        let (file_path, public_url) = self
            .storage
            .save_base64_image(&response.image_data, &response.mime_type)?;

        // Update response with storage info
        response.file_path = Some(file_path.to_string_lossy().to_string());
        response.public_url = Some(public_url.clone());
        response.file_size_bytes = Some(response.image_data.len());

        // Store in database
        self.persist_image_generation(
            &request,
            &response,
            &file_path.to_string_lossy(),
            &public_url,
        )
        .await?;

        Ok(response)
    }

    pub async fn generate_batch(
        &self,
        requests: Vec<ImageGenerationRequest>,
    ) -> Result<Vec<ImageGenerationResponse>> {
        let mut responses = Vec::new();

        for request in requests {
            match self.generate_image(request).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    return Err(e);
                },
            }
        }

        Ok(responses)
    }

    pub async fn get_generated_image(
        &self,
        uuid: &str,
    ) -> Result<Option<crate::models::image_generation::GeneratedImageRecord>> {
        self.image_repo
            .get_generated_image_by_uuid(uuid)
            .await
            .map_err(AiError::DatabaseError)
    }

    pub async fn list_user_images(
        &self,
        user_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<crate::models::image_generation::GeneratedImageRecord>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        self.image_repo
            .list_generated_images_by_user(user_id, limit, offset)
            .await
            .map_err(AiError::DatabaseError)
    }

    pub async fn delete_image(&self, uuid: &str) -> Result<()> {
        // Get image record
        let image = self.image_repo.get_generated_image_by_uuid(uuid).await?;

        if let Some(image_record) = image {
            // Delete from filesystem
            let file_path = std::path::Path::new(&image_record.file_path);
            self.storage.delete_image(file_path)?;

            // Mark as deleted in database
            self.image_repo.delete_generated_image(uuid).await?;
        }

        Ok(())
    }

    fn find_provider_for_model(&self, model: &str) -> Result<String> {
        for (name, provider) in &self.providers {
            if provider.supports_model(model) {
                return Ok(name.clone());
            }
        }

        Err(AiError::ProviderError {
            provider: "unknown".to_string(),
            message: format!("No provider found for model: {model}"),
        })
    }

    async fn persist_image_generation(
        &self,
        request: &ImageGenerationRequest,
        response: &ImageGenerationResponse,
        file_path: &str,
        public_url: &str,
    ) -> Result<()> {
        let user_id = request.user_id.as_deref().unwrap_or("anonymous");

        self.ai_request_repo
            .store_image_request(
                &response.request_id,
                user_id,
                request.session_id.as_deref(),
                request.trace_id.as_deref(),
                &response.provider,
                &response.model,
                response.cost_estimate.map(|c| c as i32),
                Some(response.generation_time_ms as i32),
                "completed",
                Some(1),
            )
            .await
            .map_err(AiError::DatabaseError)?;

        self.image_repo
            .insert_generated_image(
                &response.id,
                &response.request_id,
                &request.prompt,
                &response.model,
                &response.provider,
                file_path,
                public_url,
                response.file_size_bytes.map(|s| s as i32),
                &response.mime_type,
                Some(response.resolution.as_str()),
                Some(response.aspect_ratio.as_str()),
                Some(response.generation_time_ms as i32),
                response.cost_estimate,
                request.user_id.as_deref(),
                request.session_id.as_deref(),
                request.trace_id.as_deref(),
                None,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::providers::gemini_images::GeminiImageProvider;
    use std::sync::Arc;

    #[test]
    fn test_provider_registration() {
        let db_pool = systemprompt_core_database::DbPool::PostgreSQL(Arc::new(
            sqlx::PgPool::connect_lazy("postgresql://test").unwrap(),
        ));
        let storage_config = StorageConfig::default();

        let mut service = ImageService::new(db_pool, storage_config).unwrap();

        let provider = Arc::new(GeminiImageProvider::new("test_key".to_string()));
        service.register_provider(provider);

        assert_eq!(service.list_providers().len(), 1);
        assert!(service.get_provider("gemini-image").is_some());
    }

    #[test]
    fn test_find_provider_for_model() {
        let db_pool = systemprompt_core_database::DbPool::PostgreSQL(Arc::new(
            sqlx::PgPool::connect_lazy("postgresql://test").unwrap(),
        ));
        let storage_config = StorageConfig::default();

        let mut service = ImageService::new(db_pool, storage_config).unwrap();

        let provider = Arc::new(GeminiImageProvider::new("test_key".to_string()));
        service.register_provider(provider);

        let result = service.find_provider_for_model("gemini-2.5-flash-image");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "gemini-image");
    }
}
