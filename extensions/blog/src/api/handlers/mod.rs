//! API handlers for blog extension.
//!
//! Content is served as static HTML via the SCG pipeline.
//! This module only exports link tracking handlers.

mod content;
mod links;
mod search;

// Content handlers exist but are not used - content is served as static HTML
#[allow(unused_imports)]
pub use content::{get_content_handler, list_content_handler, query_handler};

// Link tracking handlers (active)
pub use links::{
    campaign_performance_handler, content_journey_handler, generate_link_handler,
    link_clicks_handler, link_performance_handler, list_links_handler, record_click_handler,
    redirect_handler,
};

// Search handler
pub use search::search_handler;
