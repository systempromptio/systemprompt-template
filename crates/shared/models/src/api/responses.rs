use super::pagination::PaginationInfo;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLinks {
    pub self_link: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    pub docs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub request_id: Uuid,

    pub timestamp: DateTime<Utc>,

    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseMeta {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            pagination: None,
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationInfo) -> Self {
        self.pagination = Some(pagination);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: 'static,
{
    pub data: T,

    pub meta: ResponseMeta,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResponseLinks>,
}

impl<T: Serialize + 'static> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta::new(),
            links: None,
        }
    }

    pub fn with_links(mut self, links: ResponseLinks) -> Self {
        self.links = Some(links);
        self
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = meta;
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleResponse<T>
where
    T: 'static,
{
    pub data: T,

    pub meta: ResponseMeta,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResponseLinks>,
}

impl<T: Serialize + 'static> SingleResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta::new(),
            links: None,
        }
    }

    pub const fn with_meta(data: T, meta: ResponseMeta) -> Self {
        Self {
            data,
            meta,
            links: None,
        }
    }

    pub fn with_links(mut self, links: ResponseLinks) -> Self {
        self.links = Some(links);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionResponse<T>
where
    T: 'static,
{
    pub data: Vec<T>,

    pub meta: ResponseMeta,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResponseLinks>,
}

impl<T: Serialize + 'static> CollectionResponse<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            meta: ResponseMeta::new(),
            links: None,
        }
    }

    pub fn paginated(data: Vec<T>, pagination: PaginationInfo) -> Self {
        Self {
            data,
            meta: ResponseMeta::new().with_pagination(pagination),
            links: None,
        }
    }

    pub fn with_links(mut self, links: ResponseLinks) -> Self {
        self.links = Some(links);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,

    pub meta: ResponseMeta,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            meta: ResponseMeta::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedResponse<T>
where
    T: 'static,
{
    pub data: T,

    pub meta: ResponseMeta,

    pub location: String,
}

impl<T: Serialize + 'static> CreatedResponse<T> {
    pub fn new(data: T, location: impl Into<String>) -> Self {
        Self {
            data,
            meta: ResponseMeta::new(),
            location: location.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]

pub struct AcceptedResponse {
    pub message: String,

    pub job_id: Option<String>,

    pub status_url: Option<String>,

    pub meta: ResponseMeta,
}

impl AcceptedResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            job_id: None,
            status_url: None,
            meta: ResponseMeta::new(),
        }
    }

    pub fn with_job(mut self, job_id: impl Into<String>, status_url: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self.status_url = Some(status_url.into());
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Link {
    pub fn new(href: impl Into<String>, title: Option<String>) -> Self {
        Self {
            href: href.into(),
            title,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryResponse<T>
where
    T: 'static,
{
    pub data: T,
    pub meta: ResponseMeta,
    #[serde(rename = "_links")]
    pub links: IndexMap<String, Link>,
}

impl<T: Serialize + 'static> DiscoveryResponse<T> {
    pub fn new(data: T, links: IndexMap<String, Link>) -> Self {
        Self {
            data,
            meta: ResponseMeta::new(),
            links,
        }
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = meta;
        self
    }
}
