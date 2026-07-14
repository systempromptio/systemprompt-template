//! Cost-focused aggregations over `ai_requests` for `/admin/analytics/costs`.
//!
//! - [`fetch_cost_kpis`] — single-row KPI strip (spend, requests, tokens,
//!   throughput).
//! - [`fetch_cost_by_model`] — per-(provider, model) rollup ordered by spend.
//! - [`fetch_cost_by_provider`] — per-provider rollup ordered by spend.
//! - [`fetch_token_throughput_over_time`] — 24-bucket input/output token
//!   series.

mod kpis;
mod recent;
mod rollups;
mod throughput;

pub use kpis::{CostKpis, fetch_cost_kpis};
pub use recent::{RecentRequest, fetch_recent_requests};
pub use rollups::{ModelCostRow, ProviderCostRow, fetch_cost_by_model, fetch_cost_by_provider};
pub use throughput::{ThroughputBucket, fetch_token_throughput_over_time};
