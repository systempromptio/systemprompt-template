use super::fetcher::CimdFetcher;
use crate::models::cimd::ClientValidation;
use crate::repository::oauth::OAuthRepository;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use systemprompt_core_database::Database;
use systemprompt_identifiers::ClientId;

#[derive(Debug)]
pub struct ClientValidator {
    dcr_repo: OAuthRepository,
    cimd_fetcher: CimdFetcher,
}

impl ClientValidator {
    pub fn new(db_pool: Arc<Database>) -> Self {
        Self {
            dcr_repo: OAuthRepository::new(db_pool),
            cimd_fetcher: CimdFetcher::new(),
        }
    }

    pub async fn validate_client(
        &self,
        client_id: &ClientId,
        redirect_uri: Option<&str>,
    ) -> Result<ClientValidation> {
        match client_id.client_type() {
            systemprompt_identifiers::ClientType::Cimd => {
                self.validate_cimd(client_id, redirect_uri).await
            },
            systemprompt_identifiers::ClientType::FirstParty => Ok(ClientValidation::FirstParty {
                client_id: client_id.clone(),
            }),
            systemprompt_identifiers::ClientType::ThirdParty => self.validate_dcr(client_id).await,
            systemprompt_identifiers::ClientType::System => Ok(ClientValidation::System {
                client_id: client_id.clone(),
            }),
            systemprompt_identifiers::ClientType::Unknown => Err(anyhow!(
                "Invalid client_id format: '{client_id}'. Expected patterns:\n- https://* (CIMD \
                 decentralized client)\n- sp_* (first-party SystemPrompt client)\n- client_* \
                 (third-party registered client)\n- sys_* (internal system service)"
            )),
        }
    }

    async fn validate_cimd(
        &self,
        client_id: &ClientId,
        redirect_uri: Option<&str>,
    ) -> Result<ClientValidation> {
        let metadata = self.cimd_fetcher.fetch_metadata(client_id.as_str()).await?;

        if let Some(uri) = redirect_uri {
            if !metadata.has_redirect_uri(uri) {
                return Err(anyhow!(
                    "redirect_uri '{uri}' not registered in CIMD metadata for {client_id}"
                ));
            }
        }

        Ok(ClientValidation::Cimd {
            client_id: client_id.clone(),
            metadata: Box::new(metadata),
        })
    }

    async fn validate_dcr(&self, client_id: &ClientId) -> Result<ClientValidation> {
        let client = self.dcr_repo.get_client(client_id.as_str()).await?;

        if client.is_none() {
            return Err(anyhow!(
                "DCR client_id '{client_id}' not found in oauth_clients table"
            ));
        }

        Ok(ClientValidation::Dcr {
            client_id: client_id.clone(),
        })
    }
}
