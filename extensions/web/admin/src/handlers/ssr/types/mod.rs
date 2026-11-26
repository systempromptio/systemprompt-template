//! Template context types for the SSR pages.

mod breadcrumb;
#[cfg(feature = "gateway")]
mod charts;
#[cfg(feature = "gateway")]
mod groups;
#[cfg(feature = "gateway")]
mod groups_listing;
#[cfg(any(feature = "gateway", feature = "mcp"))]
mod pickable_user;
#[cfg(feature = "gateway")]
mod pie;
#[cfg(feature = "mcp")]
mod project_report;
#[cfg(feature = "mcp")]
mod projects_page;
mod settings;
#[cfg(feature = "gateway")]
mod sparkline;
#[cfg(feature = "gateway")]
mod svg_line;
pub(crate) mod table;
mod tabs;
mod users;

pub(crate) use breadcrumb::*;
#[cfg(feature = "gateway")]
pub(crate) use charts::*;
#[cfg(feature = "gateway")]
pub(crate) use groups::*;
#[cfg(feature = "gateway")]
pub(crate) use groups_listing::*;
#[cfg(any(feature = "gateway", feature = "mcp"))]
pub(crate) use pickable_user::*;
#[cfg(feature = "gateway")]
pub(crate) use pie::*;
#[cfg(feature = "mcp")]
pub(crate) use project_report::*;
#[cfg(feature = "mcp")]
pub(crate) use projects_page::*;
pub(crate) use settings::*;
#[cfg(feature = "gateway")]
pub(crate) use sparkline::*;
#[cfg(feature = "gateway")]
pub(crate) use svg_line::*;
pub(crate) use table::*;
pub(crate) use tabs::*;
pub(crate) use users::*;
