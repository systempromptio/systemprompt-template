use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Log module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        CreateLog => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/create.sql"
        )),
        GetLog => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get.sql"
        )),
        ListLogs => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/list.sql"
        )),
        ListLogsPaginated => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/list_paginated.sql"
        )),
        DeleteLog => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/delete.sql"
        )),
        DeleteOldLogs => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/cleanup_old.sql"
        )),
        LogAnalyticsEvent => Some(include_str!(
            "../../../../log/src/queries/analytics/postgres/log_event.sql"
        )),
        GetLogsByLevel => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_by_level.sql"
        )),
        GetLogsByModule => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_by_module.sql"
        )),
        GetLogsByUser => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_by_user.sql"
        )),
        GetLogsBySession => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_by_session.sql"
        )),
        GetLogStats => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_stats.sql"
        )),
        GetErrorRate => Some(include_str!(
            "../../../../log/src/queries/logs/postgres/get_error_rate.sql"
        )),

        _ => None,
    }
}
