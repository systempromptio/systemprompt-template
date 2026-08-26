# REQ-047 / REQ-048 — Knowledge-bank MCP server (project-context migration)

Status: scaffolded, disabled by default. The server builds, registers, and
answers honestly that no retrieval backend is configured. The production
backend is a **migration of the Node.js "project-context" system** at
`github.com/Astound-Digital/sfcc-next-cursor/.project/project-context` into
this crate, behind the `KnowledgeRetriever` trait.

## What was scaffolded

- `extensions/mcp/knowledge-bank/` — workspace crate
  `systemprompt-mcp-knowledge-bank`, modeled on the `systemprompt` MCP agent:
  same auth path (`enforce_rbac_from_registry` + `record_mcp_access`
  auditing), same `McpToolExecutor`/`CliArtifact` response shape, same HTTP
  serving shell in `main.rs` (`MCP_SERVICE_ID` / `MCP_PORT`, default 5030).
- `services/mcp/knowledge-bank.yaml` — internal server, `enabled: false`,
  OAuth required, audience `mcp`, scope `admin`; listed in
  `services/config/config.yaml` includes.
- `services/access-control/roles.yaml` — `mcp_server: knowledge-bank` granted
  to role `admin` only (least privilege while the backend is a stub).
- Tests: `tests/unit/mcp/src/knowledge_bank.rs` pins the three-tool contract,
  the required-`query` / bounded-`top_k` schema, and the stub's refusal to
  fabricate results.

## Tool-contract parity with the Node server

| Node tool (`scripts/mcp/server.mjs`) | This server | Notes |
|---|---|---|
| `search(query, source_type: string[] REQUIRED, top_k?: 1..20)` | `search_knowledge(query, source_types?: string[], top_k?: 1..20)` | Same semantics: categories listed explicitly, archive categories opt-in only; `source_types` is optional only while no backend is configured — the migrated backend should enforce non-empty like the Node server. |
| `list_sources()` → source types + doc counts + last sync | `list_knowledge_sources()` | `KnowledgeSource` carries `doc_count` and `last_synced` (RFC 3339), rendered when known. |
| `index_stats()` | `knowledge_index_stats()` | `IndexStats {documents, chunks, last_built, version}`. |

## The retriever contract

`extensions/mcp/knowledge-bank/src/retriever.rs`:

```rust
#[async_trait]
pub trait KnowledgeRetriever: Send + Sync {
    async fn search(&self, query: &str, filter: &SearchFilter)
        -> Result<Vec<KnowledgeHit>, KnowledgeBankError>;
    async fn list_sources(&self) -> Result<Vec<KnowledgeSource>, KnowledgeBankError>;
    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError>;
}
```

`SearchFilter` = `source_types: Vec<String>` + `top_k: Option<u8>` (cap 20).
`KnowledgeHit` = source_type (category), source (system), title, snippet, uri
(file/line pointer back to the original), score 0..1. The server holds an
`Arc<dyn KnowledgeRetriever>`; `main.rs` currently injects `StubRetriever`,
which fails `search`/`index_stats` with `BACKEND_NOT_CONFIGURED` and lists no
sources — never fake data. A `NotConfigured` answer renders as an honest text
artifact; every other error is a tool error.

## Migration map (Node module → Rust destination)

| Node (project-context) | Rust destination | Status |
|---|---|---|
| `scripts/mcp/server.mjs` (stdio MCP) | `src/server/` + `src/tools.rs` | Ported as contract; served over authenticated HTTP, not stdio. |
| `scripts/retrieval/service.mjs` (hybrid search, candidate_multiplier over-fetch, Cohere rerank, top_k cap, per-call `.version` re-check) | new `src/retrieval/` module implementing `KnowledgeRetriever` | To build. Keep the `.version` re-check so index rebuilds are picked up without restart. |
| `scripts/retrieval/store.mjs` (LanceDB table, `source_type IN (...)` pre-filter) | vector-store layer under `src/retrieval/` | To build — store decision open, see below. |
| `scripts/ingest/atlassian.mjs`, parsers, normalizer, chunker (per-source `llm \| script \| none` strategy, LLM contextual enrichment at index time) | future offline ingestion jobs (NOT the serving path; candidates: scheduler jobs or a CLI extension) | To build. |
| `scripts/cli/index.mjs` + `sync.mjs` | same ingestion surface as above | Open decision: scheduler jobs vs. operator CLI. |
| `core/config` + env (bedrock models, chunking targets, retrieval tunables `{top_k, candidate_multiplier, rerank_enabled}`, sources map) | crate config (YAML + profile secret store for the key) | To build. |
| inbox/<source_type> directories; sources today: `meeting_notes`, `confluence` (space mirror), `jira` (issues+comments) | ingestion input layout, ported as-is or re-pointed | To decide with ingestion. |

## Bedrock specifics (from the Node system)

- Embeddings: `amazon.titan-embed-text-v2:0`, 1024-dim.
- Chunking/enrichment LLM: `openai.gpt-oss-120b-1:0` (via bedrock-runtime).
- Rerank: `cohere.rerank-v3-5:0` (via bedrock-runtime).
- Auth: `AWS_BEARER_TOKEN_BEDROCK` bearer key (not sig-v4 access keys) —
  provision via the profile secret store.
- **No gateway changes**: the AI gateway speaks no Bedrock wire protocol and
  none is needed — all Bedrock calls happen inside this MCP process (and the
  offline ingestion jobs) via the AWS SDK / bedrock-runtime HTTP.

## Open decisions

1. **Vector store**: the Node system uses LanceDB, a local file DB. Natural
   equivalents here: pgvector in the existing Postgres (fits the per-clone
   Docker/Fly Postgres story) or a ported LanceDB directory (closest to the
   source, but a file-on-disk dependency per instance). Not picked.
2. **Archive-source semantics**: Node makes archive categories opt-in-only at
   the tool boundary; decide whether that stays a tool-description contract or
   becomes enforced filtering in the retriever.
3. **Ingestion surface**: do `index`/`sync` CLIs become scheduler jobs in this
   instance, an `extensions/cli/*` extension, or stay an offline operator
   tool?
4. **`source_types` required**: flip the tool schema to required-non-empty
   once a real backend exists (parity with Node), or keep optional-with-live-
   default.

## Left for the orchestrator

- Compile validation (`just verify` / `just preflight`) — not run here by
  instruction; `.sqlx/` not needed (crate has no SQL).
- The new binary is built by `systemprompt build mcp` walking `manifest.yaml`
  (`build_type: workspace`); confirm on the next `just build-mcp`.
- Coverage baseline may need re-recording after tests run.
