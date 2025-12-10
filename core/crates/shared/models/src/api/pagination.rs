use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: i64,

    pub page: i32,

    pub per_page: i32,

    pub total_pages: i32,

    pub has_next: bool,

    pub has_prev: bool,

    pub next_url: Option<String>,

    pub prev_url: Option<String>,
}

impl PaginationInfo {
    pub fn new(total: i64, page: i32, per_page: i32) -> Self {
        let per_page_i64 = i64::from(per_page);
        let total_pages =
            i32::try_from((total + per_page_i64 - 1) / per_page_i64).unwrap_or(i32::MAX);
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            total,
            page,
            per_page,
            total_pages,
            has_next,
            has_prev,
            next_url: None,
            prev_url: None,
        }
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        if self.has_next {
            self.next_url = Some(format!(
                "{}?page={}&per_page={}",
                base_url,
                self.page + 1,
                self.per_page
            ));
        }
        if self.has_prev {
            self.prev_url = Some(format!(
                "{}?page={}&per_page={}",
                base_url,
                self.page - 1,
                self.per_page
            ));
        }
        self
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i32,

    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

const fn default_page() -> i32 {
    1
}

const fn default_per_page() -> i32 {
    20
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationParams {
    pub const fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }

    pub const fn limit(&self) -> i32 {
        self.per_page
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Asc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortParams {
    pub sort_by: Option<String>,

    #[serde(default)]
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,

    #[serde(flatten)]
    pub search: SearchQuery,
}
