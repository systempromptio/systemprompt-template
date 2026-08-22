//! Build the Metadata API deployment package for an [`OrgSpec`].
//!
//! Four components, emitted as raw XML rather than through a serializer so the
//! element order and the exact element set stay under this module's control —
//! both matter to the Metadata API.
//!
//! The element names were read back from a live org by submitting deliberately
//! invalid packages under `checkOnly` and reading the validation errors, which
//! name every rejected element. They are not guesses, and they are
//! version-specific: re-derive them when
//! [`METADATA_VERSION`](crate::services::salesforce_org::client::METADATA_VERSION)
//! moves.

use crate::services::salesforce_org::client::METADATA_VERSION;
use crate::services::salesforce_org::spec::{ExternalClientApp, OrgSpec};

const METADATA_NS: &str = "http://soap.sforce.com/2006/04/metadata";

fn xml_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn element(name: &str, value: &str) -> String {
    format!("    <{name}>{}</{name}>\n", xml_escape(value))
}

// Why: Returned as `(path_in_zip, contents)` pairs so the caller can inspect or
// print the package without deploying it — which is what makes `--dry-run`
// able to show exactly what would be sent.
#[must_use]
pub fn build_package(spec: &OrgSpec, certificate: Option<&str>) -> Vec<(String, String)> {
    let app = &spec.external_client_app;
    let name = &app.developer_name;
    let oauth_name = format!("{name}_oauth");
    let global_name = format!("{name}_glbloauth");
    let policy_name = format!("{name}_oauthPlcy");

    let package = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Package xmlns=\"{METADATA_NS}\">\n\
         {}{}{}{}    <version>{}</version>\n</Package>\n",
        types_block("ExternalClientApplication", name),
        types_block("ExtlClntAppGlobalOauthSettings", &global_name),
        types_block("ExtlClntAppOauthSettings", &oauth_name),
        types_block("ExtlClntAppOauthConfigurablePolicies", &policy_name),
        METADATA_VERSION,
    );

    vec![
        ("package.xml".to_owned(), package),
        (format!("externalClientApps/{name}.eca"), build_eca(app)),
        (
            format!("extlClntAppGlobalOauthSets/{global_name}.ecaGlblOauth"),
            build_global_oauth(app, name, &global_name, certificate),
        ),
        (
            format!("extlClntAppOauthSettings/{oauth_name}.ecaOauth"),
            build_oauth_settings(app, name, &oauth_name),
        ),
        (
            format!("extlClntAppOauthPolicies/{policy_name}.ecaOauthPlcy"),
            build_policies(app, name, &policy_name),
        ),
    ]
}

fn build_eca(app: &ExternalClientApp) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ExternalClientApplication xmlns=\"{METADATA_NS}\">\n"
    );
    out.push_str(&element("contactEmail", &app.contact_email));
    if let Some(description) = &app.description {
        out.push_str(&element("description", description));
    }
    out.push_str(&element("distributionState", &app.distribution_state));
    out.push_str(&element("label", &app.label));
    out.push_str("</ExternalClientApplication>\n");
    out
}

// Why: accepts a full PEM because that is what the operator has on disk next to
// the private key; an already-bare base64 blob passes through unchanged.
fn certificate_body(pem: &str) -> String {
    pem.lines()
        .filter(|line| !line.starts_with("-----"))
        .map(str::trim)
        .collect()
}

fn build_global_oauth(
    app: &ExternalClientApp,
    name: &str,
    label: &str,
    certificate: Option<&str>,
) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ExtlClntAppGlobalOauthSettings xmlns=\"{METADATA_NS}\">\n"
    );
    out.push_str(&element("callbackUrl", &app.oauth.callback_url));
    // Why: emitted, never omitted. `certificate` is in schema here, and a
    // declarative deploy that leaves it out clears the app's digital signature —
    // which is the credential this tool authenticates with. Omitting it once
    // cost a live org its JWT-bearer grant.
    if let Some(pem) = certificate {
        out.push_str(&element("certificate", &certificate_body(pem)));
    }
    out.push_str(&element("externalClientApplication", name));
    out.push_str(&element(
        "isConsumerSecretOptional",
        &app.oauth.consumer_secret_optional.to_string(),
    ));
    // Why: emitted explicitly rather than omitted. The deploy is declarative,
    // and this element came into schema at metadata version 67.0 — leaving it
    // out would take the default and stop the org issuing the JWT-format access
    // tokens the REST metadata deploy depends on.
    out.push_str(&element(
        "isNamedUserJwtEnabled",
        &app.oauth.named_user_jwt.to_string(),
    ));
    out.push_str(&element(
        "isPkceRequired",
        &app.oauth.pkce_required.to_string(),
    ));
    out.push_str(&element("label", label));
    out.push_str("</ExtlClntAppGlobalOauthSettings>\n");
    out
}

fn build_oauth_settings(app: &ExternalClientApp, name: &str, label: &str) -> String {
    let scopes = app
        .oauth
        .scopes
        .iter()
        .map(|s| s.metadata_token())
        .collect::<Vec<_>>()
        .join(",");
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ExtlClntAppOauthSettings xmlns=\"{METADATA_NS}\">\n"
    );
    out.push_str(&element("commaSeparatedOauthScopes", &scopes));
    out.push_str(&element("externalClientApplication", name));
    out.push_str(&element(
        "isFirstPartyAppEnabled",
        &app.oauth.first_party_app_enabled.to_string(),
    ));
    out.push_str(&element("label", label));
    if let Some(url) = &app.oauth.single_logout_url {
        out.push_str(&element("singleLogoutUrl", url));
    }
    out.push_str("</ExtlClntAppOauthSettings>\n");
    out
}

fn build_policies(app: &ExternalClientApp, name: &str, label: &str) -> String {
    let policies = &app.policies;
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ExtlClntAppOauthConfigurablePolicies xmlns=\"{METADATA_NS}\">\n"
    );
    out.push_str(&element("externalClientApplication", name));
    out.push_str(&element(
        "ipRelaxationPolicyType",
        policies.ip_relaxation.metadata_token(),
    ));
    out.push_str(&element("label", label));
    out.push_str(&element(
        "permittedUsersPolicyType",
        &policies.permitted_users,
    ));
    out.push_str(&element(
        "refreshTokenPolicyType",
        &policies.refresh_token_policy,
    ));
    if let Some(validity) = &policies.refresh_token_validity {
        out.push_str(&element(
            "refreshTokenValidityPeriod",
            &validity.period.to_string(),
        ));
        out.push_str(&element(
            "refreshTokenValidityUnit",
            validity.unit.metadata_token(),
        ));
    }
    if let Some(level) = &policies.required_session_level {
        out.push_str(&element("requiredSessionLevel", level));
    }
    out.push_str("</ExtlClntAppOauthConfigurablePolicies>\n");
    out
}

fn types_block(name: &str, member: &str) -> String {
    let mut out = String::from("    <types>\n");
    out.push_str(&format!(
        "        <members>{}</members>\n",
        xml_escape(member)
    ));
    out.push_str(&format!("        <name>{name}</name>\n    </types>\n"));
    out
}
