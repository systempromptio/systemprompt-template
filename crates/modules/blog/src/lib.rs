#![allow(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]

pub mod api;
pub mod models;
pub mod repository;
pub mod services;

pub use models::{
    Content, ContentMetadata, IngestionReport, SearchFilters, SearchRequest, SearchResponse,
    SearchResult,
};

pub use repository::{ContentRepository, SearchRepository};

pub use services::{
    ContentSourceExport, ExportService, ExportStats, GenericIngestionService, IngestionService,
    RendererService, SearchService,
};

pub use api::{get_content_handler, list_content_by_source_handler, query_handler, router};
