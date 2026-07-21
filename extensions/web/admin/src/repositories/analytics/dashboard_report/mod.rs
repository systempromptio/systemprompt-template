//! The composite analytics report shown on the reporting page.

pub mod queries;

pub use queries::{
    ContentBreakdownResult, DeviceSessionsRow, FunnelRow, GeoRow, LandingRow, SeoRow, SourceRow,
    SparkSessionRow, SparkSignupRow, TopContentRow,
};
