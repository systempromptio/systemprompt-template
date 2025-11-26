use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Users module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        CreateUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/create_user.sql"
        )),
        GetUserById => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_by_uuid.sql"
        )),
        GetUserByName => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_by_name.sql"
        )),
        GetUserByEmail => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_by_email.sql"
        )),
        UpdateUserEmail => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_email.sql"
        )),
        UpdateUserFullName => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_full_name.sql"
        )),
        UpdateUserStatus => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_status.sql"
        )),
        UpdateUserEmailFullName => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_email_full_name.sql"
        )),
        UpdateUserEmailStatus => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_email_status.sql"
        )),
        UpdateUserFullNameStatus => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_full_name_status.sql"
        )),
        UpdateUserAllFields => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_user_all_fields.sql"
        )),
        DeleteUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/delete_user.sql"
        )),
        ListUsers => Some(include_str!(
            "../../../../users/src/queries/core/postgres/list_users.sql"
        )),
        SearchUsers => Some(include_str!(
            "../../../../users/src/queries/core/postgres/search_users.sql"
        )),
        AssignRole => Some(include_str!(
            "../../../../users/src/queries/core/postgres/assign_roles.sql"
        )),
        FindByRole => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_by_role.sql"
        )),
        FindFirstAdmin => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_first_admin.sql"
        )),
        FindFirstUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/find_first_user.sql"
        )),
        CountUsers => Some(include_str!(
            "../../../../users/src/queries/core/postgres/count_users.sql"
        )),
        GetAuthenticatedUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/get_authenticated_user.sql"
        )),
        CreateAnonymousUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/create_anonymous_user.sql"
        )),
        DeleteAnonymousUser => Some(include_str!(
            "../../../../users/src/queries/core/postgres/delete_anonymous_user.sql"
        )),
        IsTemporaryAnonymous => Some(include_str!(
            "../../../../users/src/queries/core/postgres/is_temporary_anonymous.sql"
        )),
        CleanupOldAnonymousUsers => Some(include_str!(
            "../../../../users/src/queries/core/postgres/cleanup_old_anonymous_users.sql"
        )),
        ListActiveUserSessions => Some(include_str!(
            "../../../../users/src/queries/core/postgres/list_active_sessions.sql"
        )),
        ListRecentUserSessions => Some(include_str!(
            "../../../../users/src/queries/core/postgres/list_recent_sessions.sql"
        )),
        GetUserActivity => Some(include_str!(
            "../../../../users/src/queries/core/postgres/get_user_activity.sql"
        )),
        UpdateUserRoles => Some(include_str!(
            "../../../../users/src/queries/core/postgres/update_user_roles.sql"
        )),
        ListUserSessions => Some(include_str!(
            "../../../../users/src/queries/core/postgres/list_user_sessions.sql"
        )),

        _ => None,
    }
}
