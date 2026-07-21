//! The composite analytics report shown on the reporting page.

pub mod queries;

pub use queries::{
    ContentBreakdownResult, DeviceRow, FunnelRow, GeoRow, LandingRow, SeoRow, SourceRow,
    SparkSessionRow, SparkSignupRow, TopContentRow,
};
