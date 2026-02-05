#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unchecked_duration_subtraction)]
#![allow(clippy::default_trait_access)]

pub mod api;
pub mod client;
pub mod error;
pub mod extension;
pub mod jobs;
pub mod models;
pub mod security;

pub use client::MoltbookClient;
pub use error::MoltbookError;
pub use extension::MoltbookExtension;

pub use models::{
    AgentProfile, CreateCommentRequest, CreateCommentResponse, CreatePostRequest,
    CreatePostResponse, ListCommentsQuery, ListPostsQuery, MoltbookAgent, MoltbookComment,
    MoltbookPost, PostSearchQuery, RegisterAgentRequest, RegisterAgentResponse, Submolt,
    SubmoltSearchQuery, VoteDirection, VoteRequest,
};
