//! Template context types for the SSR pages.

mod breadcrumb;
mod charts;
mod settings;
mod table;
mod tabs;
mod users;

pub(crate) use breadcrumb::*;
pub(crate) use charts::*;
pub(crate) use settings::*;
pub(crate) use table::*;
pub(crate) use tabs::*;
pub(crate) use users::*;

mod pie;
pub(crate) use pie::*;

mod svg_line;
pub(crate) use svg_line::*;

mod svg_stack;
pub(crate) use svg_stack::*;

pub(crate) mod groups;
pub(crate) use groups::*;

pub(crate) mod groups_listing;
pub(crate) use groups_listing::*;

pub(crate) mod projects_page;
pub(crate) use projects_page::*;
