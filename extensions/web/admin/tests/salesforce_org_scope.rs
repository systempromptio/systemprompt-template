//! `salesforce_org::scope` — the three-way OAuth scope vocabulary.
//!
//! The same scope has a YAML name, a Metadata API token and an sObject column,
//! and none of the three are derivable from the others. A wrong metadata token
//! fails a deploy loudly, but a wrong sObject column is worse: export reads it
//! as `false`, diff sees the scope missing, and the next apply grants or
//! revokes something nobody asked for. These tests pin all three mappings and
//! their mutual consistency.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use std::collections::BTreeSet;

use systemprompt_web_admin::salesforce_org::scope::{OauthScope, UNMAPPED_SCOPE_FIELDS};

#[test]
fn the_vocabulary_is_the_probed_size() {
    assert_eq!(
        OauthScope::all().len(),
        36,
        "adding a scope means re-deriving its metadata token and sObject column"
    );
}

#[test]
fn no_scope_appears_twice() {
    let unique: BTreeSet<_> = OauthScope::all().iter().copied().collect();
    assert_eq!(unique.len(), OauthScope::all().len());
}

#[test]
fn every_metadata_token_is_distinct() {
    let unique: BTreeSet<_> = OauthScope::all()
        .iter()
        .map(|s| s.metadata_token())
        .collect();
    assert_eq!(unique.len(), OauthScope::all().len());
}

#[test]
fn every_sobject_field_is_distinct() {
    let unique: BTreeSet<_> = OauthScope::all()
        .iter()
        .map(|s| s.sobject_field())
        .collect();
    assert_eq!(unique.len(), OauthScope::all().len());
}

#[test]
fn no_token_or_field_is_blank() {
    for scope in OauthScope::all() {
        assert!(!scope.metadata_token().is_empty(), "{scope:?}");
        assert!(!scope.sobject_field().is_empty(), "{scope:?}");
    }
}

// Every scope column on `ExtlClntAppOauthSettings` shares this prefix. A
// mapping that does not is a typo that would silently read as `false`.
#[test]
fn every_sobject_field_is_a_scope_column() {
    for scope in OauthScope::all() {
        assert!(
            scope.sobject_field().starts_with("OauthScopes"),
            "{scope:?} maps to {}",
            scope.sobject_field()
        );
    }
}

// The mappings that are not a straight uppercase of the variant name. These
// were read back from a live org and are the ones a refactor would get wrong.
#[test]
fn the_irregular_mappings_are_pinned() {
    let cases = [
        (OauthScope::Basic, "Basic", "OauthScopesSSO"),
        (OauthScope::Mcp, "MCP", "OauthScopesMCP_API"),
        (OauthScope::OpenId, "OpenID", "OauthScopesOPENID"),
        (
            OauthScope::EinsteinGpt,
            "EinsteinGPT",
            "OauthScopesEINSTEIN_GPT_API",
        ),
        (
            OauthScope::SfApiPlatform,
            "SFApiPlatform",
            "OauthScopesSFAP_API",
        ),
        (OauthScope::Scrt, "SCRT", "OauthScopesSCRT_API"),
        (
            OauthScope::CustomApplications,
            "CustomApplications",
            "OauthScopesVF",
        ),
        (
            OauthScope::Chatter,
            "Chatter",
            "OauthScopesCHATTER_REST_API",
        ),
        (OauthScope::Wave, "Wave", "OauthScopesWAVE_REST_API"),
        (OauthScope::Eclair, "Eclair", "OauthScopesECLAIR_REST_API"),
        (
            OauthScope::CdpIdentityResolution,
            "CDPIdentityResolution",
            "OauthScopesCDP_IDENTITYRESOLUTION_API",
        ),
        (
            OauthScope::CdpCalculatedInsight,
            "CDPCalculatedInsight",
            "OauthScopesCDP_CALCULATED_INSIGHT_API",
        ),
        (
            OauthScope::DataCloudUserClaims,
            "DataCloudUserClaims",
            "OauthScopesDATA_CLOUD_USER_CLAIMS",
        ),
    ];
    for (scope, token, field) in cases {
        assert_eq!(scope.metadata_token(), token, "{scope:?} metadata token");
        assert_eq!(scope.sobject_field(), field, "{scope:?} sObject column");
    }
}

#[test]
fn the_everyday_scopes_map_as_expected() {
    let cases = [
        (OauthScope::Api, "Api", "OauthScopesAPI"),
        (OauthScope::Web, "Web", "OauthScopesWEB"),
        (OauthScope::Full, "Full", "OauthScopesFULL"),
        (
            OauthScope::RefreshToken,
            "RefreshToken",
            "OauthScopesREFRESH_TOKEN",
        ),
        (
            OauthScope::OfflineAccess,
            "OfflineAccess",
            "OauthScopesOFFLINE_ACCESS",
        ),
        (OauthScope::Profile, "Profile", "OauthScopesPROFILE"),
        (OauthScope::Email, "Email", "OauthScopesEMAIL"),
        (OauthScope::Address, "Address", "OauthScopesADDRESS"),
        (OauthScope::Phone, "Phone", "OauthScopesPHONE"),
        (
            OauthScope::CustomPermissions,
            "CustomPermissions",
            "OauthScopesCUSTOM_PERMISSIONS",
        ),
        (OauthScope::Content, "Content", "OauthScopesCONTENT"),
        (OauthScope::Lightning, "Lightning", "OauthScopesLIGHTNING"),
        (OauthScope::Cdp, "CDP", "OauthScopesCDP_API"),
        (OauthScope::CdpQuery, "CDPQuery", "OauthScopesCDP_QUERY_API"),
    ];
    for (scope, token, field) in cases {
        assert_eq!(scope.metadata_token(), token, "{scope:?} metadata token");
        assert_eq!(scope.sobject_field(), field, "{scope:?} sObject column");
    }
}

// Export drives its SOQL off this, so a column missing from the projection is
// a scope that always reads back as absent.
#[test]
fn the_projection_lists_every_column() {
    let projection = OauthScope::soql_projection();
    for scope in OauthScope::all() {
        assert!(
            projection.contains(scope.sobject_field()),
            "projection omits {}",
            scope.sobject_field()
        );
    }
}

#[test]
fn the_projection_is_a_bare_comma_separated_list() {
    let projection = OauthScope::soql_projection();
    assert!(
        !projection.contains(' '),
        "SOQL list must not pad: {projection}"
    );
    assert!(!projection.starts_with(','));
    assert!(!projection.ends_with(','));
    assert_eq!(
        projection.split(',').count(),
        OauthScope::all().len(),
        "{projection}"
    );
}

#[test]
fn the_projection_entries_are_the_columns_in_declared_order() {
    let projection = OauthScope::soql_projection();
    let expected: Vec<&str> = OauthScope::all()
        .iter()
        .map(|s| s.sobject_field())
        .collect();
    assert_eq!(projection.split(',').collect::<Vec<_>>(), expected);
}

// `OauthScopesHUB_API` has no metadata token, so it cannot round-trip.
// Querying it would invite export to represent something apply then clears.
#[test]
fn the_unmapped_column_is_kept_out_of_the_projection() {
    let projection = OauthScope::soql_projection();
    for field in UNMAPPED_SCOPE_FIELDS {
        assert!(
            !projection.contains(field),
            "{field} has no metadata token and must not be projected"
        );
    }
}

#[test]
fn the_unmapped_column_is_not_a_mapped_scope() {
    let mapped: BTreeSet<_> = OauthScope::all()
        .iter()
        .map(|s| s.sobject_field())
        .collect();
    for field in UNMAPPED_SCOPE_FIELDS {
        assert!(!mapped.contains(field), "{field} is mapped after all");
    }
    assert_eq!(UNMAPPED_SCOPE_FIELDS, &["OauthScopesHUB_API"]);
}

#[test]
fn scopes_round_trip_through_their_snake_case_names() {
    for scope in OauthScope::all() {
        let json = serde_json::to_string(scope).expect("serialises");
        let back: OauthScope = serde_json::from_str(&json).expect("parses");
        assert_eq!(back, *scope);
        assert!(
            !json.contains(char::is_uppercase),
            "the YAML vocabulary is snake_case: {json}"
        );
    }
}

#[test]
fn the_snake_case_names_are_the_documented_ones() {
    let cases = [
        (OauthScope::Mcp, "\"mcp\""),
        (OauthScope::Basic, "\"basic\""),
        (OauthScope::OpenId, "\"open_id\""),
        (OauthScope::RefreshToken, "\"refresh_token\""),
        (OauthScope::EinsteinGpt, "\"einstein_gpt\""),
        (OauthScope::SfApiPlatform, "\"sf_api_platform\""),
        (
            OauthScope::CdpIdentityResolution,
            "\"cdp_identity_resolution\"",
        ),
    ];
    for (scope, name) in cases {
        assert_eq!(serde_json::to_string(&scope).expect("serialises"), name);
    }
}

// Diff sorts scope sets before comparing, so the ordering must be total and
// stable rather than incidental.
#[test]
fn scopes_order_by_declaration() {
    let mut scopes = vec![OauthScope::Mcp, OauthScope::Basic, OauthScope::Api];
    scopes.sort_unstable();
    assert_eq!(
        scopes,
        vec![OauthScope::Basic, OauthScope::Api, OauthScope::Mcp]
    );
}

#[test]
fn all_is_sorted_by_its_own_ordering() {
    let mut sorted = OauthScope::all().to_vec();
    sorted.sort_unstable();
    assert_eq!(sorted, OauthScope::all());
}
