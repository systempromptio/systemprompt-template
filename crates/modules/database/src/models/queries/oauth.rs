use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Oauth module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        InsertClient => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_client.sql"
        )),
        InsertClientBase => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_client_base.sql"
        )),
        GetClientByClientId => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/get_client_base.sql"
        )),
        UpdateClient => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/update_client.sql"
        )),
        DeleteClient => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_client.sql"
        )),
        ActivateClient => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/activate_client.sql"
        )),
        DeactivateClient => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/deactivate_client.sql"
        )),
        ListClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/list_clients_base.sql"
        )),
        CountClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/count_clients.sql"
        )),
        InsertRedirectUri => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_redirect_uri.sql"
        )),
        LoadRedirectUris => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/load_redirect_uris.sql"
        )),
        DeleteRedirectUris => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_redirect_uris.sql"
        )),
        InsertGrantType => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_grant_type.sql"
        )),
        LoadGrantTypes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/load_grant_types.sql"
        )),
        DeleteGrantTypes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_grant_types.sql"
        )),
        InsertResponseType => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_response_type.sql"
        )),
        LoadResponseTypes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/load_response_types.sql"
        )),
        DeleteResponseTypes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_response_types.sql"
        )),
        InsertScope => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_scope.sql"
        )),
        LoadScopes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/load_scopes.sql"
        )),
        DeleteScopes => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_scopes.sql"
        )),
        InsertContact => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/insert_contact.sql"
        )),
        LoadContacts => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/load_contacts.sql"
        )),
        DeleteContacts => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_contacts.sql"
        )),
        DeleteUnusedClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_unused_clients.sql"
        )),
        ListUnusedClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/list_unused_clients.sql"
        )),
        ListStaleClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/list_stale_clients.sql"
        )),
        DeactivateOldTestClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/deactivate_old_test_clients.sql"
        )),
        InsertAuthorizationCode => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/insert_auth_code.sql"
        )),
        GetAuthorizationCode => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/get_auth_code_details.sql"
        )),
        CountRecentSessions => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/count_recent_sessions.sql"
        )),
        InsertCredential => Some(include_str!(
            "../../../../oauth/src/queries/webauthn/postgres/insert_credential.sql"
        )),
        GetCredentialsByUserId => Some(include_str!(
            "../../../../oauth/src/queries/webauthn/postgres/get_credentials.sql"
        )),
        UpdateCredentialCounter => Some(include_str!(
            "../../../../oauth/src/queries/webauthn/postgres/update_credential_counter.sql"
        )),
        GetClientAnalytics => Some(include_str!(
            "../../../../oauth/src/queries/analytics/postgres/get_client_analytics.sql"
        )),
        GetClientAnalyticsById => Some(include_str!(
            "../../../../oauth/src/queries/analytics/postgres/get_client_analytics_by_id.sql"
        )),
        GetClientErrors => Some(include_str!(
            "../../../../oauth/src/queries/analytics/postgres/get_client_errors.sql"
        )),
        GetClientErrorsById => Some(include_str!(
            "../../../../oauth/src/queries/analytics/postgres/get_client_errors_by_id.sql"
        )),
        DeleteInactiveClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_inactive_clients.sql"
        )),
        DeleteOldTestClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/delete_old_test_clients.sql"
        )),
        ListInactiveClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/list_inactive_clients.sql"
        )),
        ListOldClients => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/list_old_clients.sql"
        )),
        UpdateClientSecret => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/update_client_secret.sql"
        )),
        UpdateClientLastUsed => Some(include_str!(
            "../../../../oauth/src/queries/clients/postgres/update_last_used.sql"
        )),
        GetAuthorizationCodeClientId => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/get_auth_code_client_id.sql"
        )),
        MarkAuthorizationCodeUsed => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/mark_auth_code_used.sql"
        )),
        InsertRefreshToken => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/insert_refresh_token.sql"
        )),
        GetRefreshToken => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/get_refresh_token.sql"
        )),
        RevokeRefreshToken => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/delete_refresh_token.sql"
        )),
        DeleteExpiredRefreshTokens => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/delete_expired_refresh_tokens.sql"
        )),
        GetRoles => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/get_roles.sql"
        )),
        CheckRoleExists => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/check_role_exists.sql"
        )),
        GetDefaultRoles => Some(include_str!(
            "../../../../oauth/src/queries/oauth/postgres/get_default_roles.sql"
        )),

        _ => None,
    }
}
