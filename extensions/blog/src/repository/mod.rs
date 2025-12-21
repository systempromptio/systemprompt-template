//! Blog extension database repositories.

pub mod content;
pub mod link;
pub mod search;

pub use content::ContentRepository;
pub use link::{LinkAnalyticsRepository, LinkRepository};
pub use search::SearchRepository;
