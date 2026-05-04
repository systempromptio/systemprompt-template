pub mod funnel;
pub mod overview;
pub mod queries;
pub mod report;

pub use funnel::{fetch_funnel_and_sparklines, FunnelSparklineResult};
pub use overview::{fetch_overview_data, AcquisitionRow, OverviewRows, PageViewsRow, SessionsRow};
pub use queries::{
    fetch_content_and_breakdown_data, ContentBreakdownResult, DeviceRow, FunnelRow, GeoRow,
    LandingRow, SeoRow, SourceRow, SparkSessionRow, SparkSignupRow, TopContentRow,
};
pub use report::upsert_traffic_report;
