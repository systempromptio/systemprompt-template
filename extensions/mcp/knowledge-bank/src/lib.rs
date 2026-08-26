//! MCP server crate for the Astound knowledge bank (REQ-047/048).
//!
//! Exposes enterprise knowledge retrieval as MCP tools (`search_knowledge`,
//! `list_knowledge_sources`, `knowledge_index_stats`) over a pluggable
//! backend: [`retriever`] defines the [`KnowledgeRetriever`] trait, and the
//! server holds a trait object so the production backend drops in without
//! touching the tool surface. That backend is a MIGRATION of the Node.js
//! "project-context" system (sfcc-next-cursor/.project/project-context),
//! whose modules map onto this crate as follows:
//!
//! - `scripts/mcp/server.mjs` (stdio MCP server) → [`server`] + [`tools`] here
//!   — already ported as the tool contract (search / `list_sources` /
//!   `index_stats`), served over this instance's authenticated HTTP transport
//!   instead of stdio.
//! - `scripts/retrieval/service.mjs` + `store.mjs` (hybrid search: Titan v2
//!   embeddings + text match, `source_type` pre-filter, candidate over-fetch,
//!   Cohere rerank, per-call index `.version` re-check) → a future `retrieval`
//!   module implementing [`KnowledgeRetriever`] via the AWS SDK. The vector
//!   store is an open decision: the Node system uses `LanceDB` (a local file
//!   DB); the natural equivalents here are pgvector in the existing Postgres or
//!   a ported `LanceDB` directory.
//! - `scripts/ingest/*` (atlassian.mjs, parsers, normalizer, chunker with
//!   per-source llm | script | none strategies, LLM contextual enrichment) plus
//!   `scripts/cli/{index,sync}.mjs` → future offline ingestion jobs. Ingestion
//!   is NOT part of the serving path and stays out of the trait.
//! - `core/config` + env (bedrock models, chunking targets, retrieval tunables,
//!   sources map) → server-side config for this crate.
//!
//! Until the migration lands, the shipped [`StubRetriever`] reports honestly
//! that no backend is configured — it never fabricates results. Errors
//! normalise on [`error::KnowledgeBankError`]; the `main` binary mirrors the
//! `systemprompt` MCP agent's HTTP serving shell. Full migration map:
//! `docs/tickets/req-047-knowledge-bank-mcp.md`.

pub mod error;
pub mod retriever;
pub mod server;
pub mod tools;

pub use retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter, StubRetriever,
};
pub use server::KnowledgeBankServer;
