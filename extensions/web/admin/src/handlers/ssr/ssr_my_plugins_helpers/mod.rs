mod aggregate;
mod associations;
mod edit_data;

pub(in crate::handlers::ssr) use aggregate::collect_my_plugins;
pub(in crate::handlers::ssr) use associations::build_association_lists;
pub(in crate::handlers::ssr) use edit_data::build_plugin_edit_data;
