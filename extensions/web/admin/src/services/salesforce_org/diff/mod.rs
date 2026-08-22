//! Compare a desired [`OrgSpec`] against an org's actual one.
//!
//! A pure function over two specs, so it is testable without an org.
//!
//! Fields that no API can read back are reported as
//! [`ChangeKind::AlwaysApplied`] rather than folded into "no changes". Calling
//! them unchanged would be a claim the tool cannot support: it has not read
//! them and does not know.
//!
//! The reporting vocabulary lives in the private `change` module.

mod change;

pub use change::{Change, ChangeKind, ChangeSet};

use change::{compare_str, push};

use super::spec::{ExternalClientApp, HostedMcpServer, OrgSpec, PermissionSetSpec, Validity};

#[must_use]
pub fn diff(actual: &OrgSpec, desired: &OrgSpec) -> ChangeSet {
    let mut changes = Vec::new();
    let (a, d) = (&actual.external_client_app, &desired.external_client_app);

    compare_str(
        &mut changes,
        "external_client_app.developer_name",
        &a.developer_name,
        &d.developer_name,
    );
    compare_str(
        &mut changes,
        "external_client_app.label",
        &a.label,
        &d.label,
    );
    compare_str(
        &mut changes,
        "external_client_app.description",
        a.description.as_deref().unwrap_or(""),
        d.description.as_deref().unwrap_or(""),
    );
    compare_str(
        &mut changes,
        "external_client_app.contact_email",
        &a.contact_email,
        &d.contact_email,
    );
    compare_str(
        &mut changes,
        "external_client_app.distribution_state",
        &a.distribution_state,
        &d.distribution_state,
    );

    diff_scopes(&mut changes, a, d);
    diff_policies(&mut changes, a, d);

    // Why: not readable from any API, so it is deployed unconditionally.
    push(
        &mut changes,
        ChangeKind::AlwaysApplied,
        "external_client_app.oauth.callback_url",
        "",
        &d.oauth.callback_url,
    );
    push(
        &mut changes,
        ChangeKind::AlwaysApplied,
        "external_client_app.oauth.pkce_required",
        "",
        &d.oauth.pkce_required.to_string(),
    );
    push(
        &mut changes,
        ChangeKind::AlwaysApplied,
        "external_client_app.oauth.consumer_secret_optional",
        "",
        &d.oauth.consumer_secret_optional.to_string(),
    );
    push(
        &mut changes,
        ChangeKind::AlwaysApplied,
        "external_client_app.oauth.named_user_jwt",
        "",
        &d.oauth.named_user_jwt.to_string(),
    );

    diff_permission_sets(
        &mut changes,
        &actual.permission_sets,
        &desired.permission_sets,
    );
    diff_hosted_mcp_servers(
        &mut changes,
        &actual.hosted_mcp_servers,
        &desired.hosted_mcp_servers,
    );

    ChangeSet { changes }
}

// Why: servers active in the org but absent from the spec are deliberately not
// reported. Apply is additive and would never deactivate one, so calling it
// drift would be reporting a difference nothing will ever resolve.
fn diff_hosted_mcp_servers(
    changes: &mut Vec<Change>,
    actual: &[HostedMcpServer],
    desired: &[HostedMcpServer],
) {
    for want in desired {
        let path = format!("hosted_mcp_servers.{}", want.developer_name);
        match actual
            .iter()
            .find(|a| a.developer_name == want.developer_name)
        {
            // Why: absent means the org does not offer this server. Apply
            // cannot fix it, so it is surfaced as an add the operator must
            // resolve rather than something the tool will silently create.
            None => push(changes, ChangeKind::Add, &path, "absent", "present"),
            Some(have) => {
                if have.active != want.active {
                    push(
                        changes,
                        ChangeKind::Update,
                        &format!("{path}.active"),
                        &have.active.to_string(),
                        &want.active.to_string(),
                    );
                }
            },
        }
    }
}

fn diff_scopes(changes: &mut Vec<Change>, actual: &ExternalClientApp, desired: &ExternalClientApp) {
    let mut have = actual.oauth.scopes.clone();
    let mut want = desired.oauth.scopes.clone();
    have.sort_unstable();
    want.sort_unstable();

    for scope in &want {
        if !have.contains(scope) {
            push(
                changes,
                ChangeKind::Add,
                "external_client_app.oauth.scopes",
                "",
                scope.metadata_token(),
            );
        }
    }
    for scope in &have {
        if !want.contains(scope) {
            push(
                changes,
                ChangeKind::Remove,
                "external_client_app.oauth.scopes",
                scope.metadata_token(),
                "",
            );
        }
    }
    if actual.oauth.first_party_app_enabled != desired.oauth.first_party_app_enabled {
        push(
            changes,
            ChangeKind::Update,
            "external_client_app.oauth.first_party_app_enabled",
            &actual.oauth.first_party_app_enabled.to_string(),
            &desired.oauth.first_party_app_enabled.to_string(),
        );
    }
}

fn diff_policies(
    changes: &mut Vec<Change>,
    actual: &ExternalClientApp,
    desired: &ExternalClientApp,
) {
    let (a, d) = (&actual.policies, &desired.policies);
    compare_str(
        changes,
        "external_client_app.policies.permitted_users",
        &a.permitted_users,
        &d.permitted_users,
    );
    compare_str(
        changes,
        "external_client_app.policies.ip_relaxation",
        a.ip_relaxation.metadata_token(),
        d.ip_relaxation.metadata_token(),
    );
    compare_str(
        changes,
        "external_client_app.policies.refresh_token_policy",
        &a.refresh_token_policy,
        &d.refresh_token_policy,
    );
    let fmt_validity = |v: Option<&Validity>| {
        v.map_or_else(
            || "none".to_owned(),
            |v| format!("{} {}", v.period, v.unit.metadata_token()),
        )
    };
    compare_str(
        changes,
        "external_client_app.policies.refresh_token_validity",
        &fmt_validity(a.refresh_token_validity.as_ref()),
        &fmt_validity(d.refresh_token_validity.as_ref()),
    );
    compare_str(
        changes,
        "external_client_app.policies.required_session_level",
        a.required_session_level.as_deref().unwrap_or(""),
        d.required_session_level.as_deref().unwrap_or(""),
    );
}

fn diff_permission_sets(
    changes: &mut Vec<Change>,
    actual: &[PermissionSetSpec],
    desired: &[PermissionSetSpec],
) {
    for want in desired {
        match actual.iter().find(|a| a.name == want.name) {
            None => push(changes, ChangeKind::Add, "permission_sets", "", &want.name),
            Some(have) => {
                compare_str(
                    changes,
                    &format!("permission_sets.{}.label", want.name),
                    &have.label,
                    &want.label,
                );
                compare_str(
                    changes,
                    &format!("permission_sets.{}.grants_app", want.name),
                    have.grants_app.as_deref().unwrap_or(""),
                    want.grants_app.as_deref().unwrap_or(""),
                );
            },
        }
    }
    for have in actual {
        if !desired.iter().any(|d| d.name == have.name) {
            push(
                changes,
                ChangeKind::Remove,
                "permission_sets",
                &have.name,
                "",
            );
        }
    }
}
