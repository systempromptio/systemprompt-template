//! Department repository: record lifecycle, dashboard rollups, and the
//! cross-user aggregates that back the user-management views.

mod aggregates;
mod crud;
mod summaries;

pub use aggregates::{
    UserManagementAggregate, UserMarketplaceOverride, list_user_management_aggregates,
    list_user_marketplace_overrides,
};
pub use crud::{
    assign_user_to_department, create_department, delete_department, get_department,
    get_department_by_name, update_department,
};
pub use summaries::{list_department_members, list_department_top_tools, list_departments};
