//! The retrieval backend contract and the shipped stub implementation.
//!
//! [`KnowledgeRetriever`] is the seam the server is built against. The
//! production implementation is a MIGRATION of the Node.js "project-context"
//! system (sfcc-next-cursor/.project/project-context): hybrid search — Bedrock
//! Titan v2 embeddings (1024-dim) plus text match over a vector store,
//! `source_type IN (...)` pre-filter, candidate over-fetch, Cohere rerank —
//! called through the AWS SDK inside this process (the AI gateway carries no
//! Bedrock wire protocol and needs no changes). It lands here once AWS
//! credentials are provisioned; see the migration map in
//! `docs/tickets/req-047-knowledge-bank-mcp.md`. Until then [`StubRetriever`]
//! keeps the server honest: it refuses to answer rather than fabricate
//! results.

use crate::error::KnowledgeBankError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Why: ceiling on `top_k`, mirroring the Node retrieval service's cap.
pub const MAX_TOP_K: u8 = 20;

/// Search constraints passed alongside the query.
///
/// `source_types` names the source categories to search (e.g.
/// `meeting_notes`, `confluence`, `jira`); empty means every live source —
/// archive categories are opt-in only and must be listed explicitly. `top_k`
/// caps results at [`MAX_TOP_K`]; `None` takes the backend's configured
/// default.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub source_types: Vec<String>,
    pub top_k: Option<u8>,
}

/// One retrieval result.
///
/// `source_type` is the category it came from, `source` names the
/// originating system, `uri` is the file/line pointer (or canonical link)
/// back to the original so agents can open it, and `score` is relevance in
/// `[0.0, 1.0]`, higher is better.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHit {
    pub source_type: String,
    pub source: String,
    pub title: String,
    pub snippet: String,
    pub uri: String,
    pub score: f64,
}

/// A knowledge source category the backend can search.
///
/// `id` is the stable value used in [`SearchFilter::source_types`];
/// `doc_count` and `last_synced` (RFC 3339) are populated when the index
/// knows them; `available` says whether the source is reachable and indexed
/// right now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub available: bool,
    pub doc_count: Option<u64>,
    pub last_synced: Option<String>,
}

/// Statistics for the retrieval index.
///
/// Totals, when the index was last built (RFC 3339), and its `version` stamp
/// — the Node system re-checks the stamp per call so a rebuild is picked up
/// without a restart, and the ported backend keeps that contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub documents: u64,
    pub chunks: u64,
    pub last_built: Option<String>,
    pub version: Option<String>,
}

/// The backend seam: everything the MCP tools need from retrieval.
///
/// Ingestion (parsers, chunking, enrichment) is deliberately NOT part of
/// this trait — it is an offline concern that feeds the index the retriever
/// reads.
#[async_trait]
pub trait KnowledgeRetriever: Send + Sync {
    async fn search(
        &self,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<KnowledgeHit>, KnowledgeBankError>;

    async fn list_sources(&self) -> Result<Vec<KnowledgeSource>, KnowledgeBankError>;

    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError>;
}

pub const NOT_CONFIGURED_MESSAGE: &str = "no retrieval backend is configured on this instance; \
                                          the Bedrock-backed project-context migration is pending \
                                          credential provisioning (REQ-047)";

/// Placeholder backend shipped until the project-context migration lands.
///
/// It is deliberately empty-handed: `search` and `index_stats` fail with
/// [`KnowledgeBankError::NotConfigured`] and `list_sources` returns no
/// sources, so no caller can mistake scaffolding for a working knowledge
/// bank.
#[derive(Debug, Clone, Copy, Default)]
pub struct StubRetriever;

#[async_trait]
impl KnowledgeRetriever for StubRetriever {
    async fn search(
        &self,
        _query: &str,
        _filter: &SearchFilter,
    ) -> Result<Vec<KnowledgeHit>, KnowledgeBankError> {
        Err(KnowledgeBankError::NotConfigured(
            NOT_CONFIGURED_MESSAGE.to_owned(),
        ))
    }

    async fn list_sources(&self) -> Result<Vec<KnowledgeSource>, KnowledgeBankError> {
        Ok(Vec::new())
    }

    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError> {
        Err(KnowledgeBankError::NotConfigured(
            NOT_CONFIGURED_MESSAGE.to_owned(),
        ))
    }
}
