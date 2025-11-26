use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{ClientId, ClientType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimdMetadata {
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<String>>,
}

impl CimdMetadata {
    pub fn validate(&self) -> Result<()> {
        if !self.client_id.starts_with("https://") {
            return Err(anyhow!("client_id must be HTTPS URL"));
        }

        if self.redirect_uris.is_empty() {
            return Err(anyhow!("redirect_uris cannot be empty"));
        }

        for uri in &self.redirect_uris {
            if uri.contains("..") || uri.contains('\0') {
                return Err(anyhow!("Invalid redirect_uri: {uri}"));
            }
        }

        Ok(())
    }

    pub fn has_redirect_uri(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|u| u == uri)
    }
}

#[derive(Debug)]
pub enum ClientValidation {
    Dcr {
        client_id: ClientId,
    },
    Cimd {
        client_id: ClientId,
        metadata: Box<CimdMetadata>,
    },
    FirstParty {
        client_id: ClientId,
    },
    System {
        client_id: ClientId,
    },
}

impl ClientValidation {
    pub const fn client_id(&self) -> &ClientId {
        match self {
            Self::Cimd { client_id, .. }
            | Self::Dcr { client_id }
            | Self::FirstParty { client_id }
            | Self::System { client_id } => client_id,
        }
    }

    pub fn client_type(&self) -> ClientType {
        match self {
            Self::Dcr { client_id } => client_id.client_type(),
            Self::Cimd { .. } => ClientType::Cimd,
            Self::FirstParty { .. } => ClientType::FirstParty,
            Self::System { .. } => ClientType::System,
        }
    }
}
