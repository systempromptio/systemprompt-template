//! Unit tests for the MCP extension crates' pure helpers:
//! - `systemprompt-mcp-agent`'s `filter_hallucinated_args` (CLI arg scrubbing)
//! - `systemprompt-mcp-shared`'s `truncate_on_char_boundary` (rejection-reason
//!   truncation with UTF-8 safety) and `AuditMetadata`'s stored JSON shape
//! - `systemprompt-mcp-agent`'s `systemprompt` tool contract (single tool, its
//!   input/output schema) and its error type's code / status / retryability
//! - `systemprompt-mcp-knowledge-bank`'s tool contract (two tools, schemas)
//!   and the honesty of its `StubRetriever` (not-configured, never fake data)

#[cfg(test)]
mod audit_metadata;
#[cfg(test)]
mod filter_hallucinated_args;
#[cfg(test)]
mod knowledge_bank;
#[cfg(test)]
mod systemprompt_error;
#[cfg(test)]
mod systemprompt_tools;
#[cfg(test)]
mod truncate_on_char_boundary;
