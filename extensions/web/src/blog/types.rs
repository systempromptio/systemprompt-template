use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub category: Option<String>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RelatedPost {
    pub slug: String,
    pub title: String,
}
