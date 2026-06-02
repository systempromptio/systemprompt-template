//! Cost-focused aggregations over `ai_requests` for `/admin/analytics/costs`.
//!
//! - [`fetch_cost_kpis`] — single-row KPI strip (spend, requests, tokens, throughput).
//! - [`fetch_cost_by_model`] — per-(provider, model) rollup ordered by spend.
//! - [`fetch_cost_by_provider`] — per-provider rollup ordered by spend.
//! - [`fetch_token_throughput_over_time`] — 24-bucket input/output token series.

mod kpis;
mod recent;
mod rollups;
mod throughput;

pub use kpis::{fetch_cost_kpis, CostKpis};
pub use recent::{fetch_recent_requests, RecentRequest};
pub use rollups::{fetch_cost_by_model, fetch_cost_by_provider, ModelCostRow, ProviderCostRow};
pub use throughput::{fetch_token_throughput_over_time, ThroughputBucket};
