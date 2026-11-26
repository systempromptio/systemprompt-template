//! The `adfs` SSO provider against a local identity provider that signs its
//! assertion with a fixture key: the AuthnRequest the provider sends is the
//! one the IdP consumes, and the posted response it verifies is the one the
//! IdP signed. A tampered response, a response answering another request and
//! an unsolicited response are refused.

use std::io::Read;
use std::time::{Duration, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use flate2::read::DeflateDecoder;
use saml::attribute::Attribute;
use saml::authn_context::AuthnContextClassRef;
use saml::binding::{Binding, Endpoint, SsoResponseDispatch};
use saml::crypto::cert::X509Certificate;
use saml::crypto::keypair::KeyPair;
use saml::descriptor::SpDescriptor;
use saml::dsig::algorithms::{
    C14nAlgorithm, DigestAlgorithm, PeerCryptoPolicy, SignatureAlgorithm,
};
use saml::idp::{ConsumeAuthnRequest, IdentityProvider, IdentityProviderConfig, IssueResponse};
use saml::nameid::{NameId, NameIdFormat};
use systemprompt::traits::sso::{
    SsoError, SsoProvider, SsoReauth, SsoReturn, SsoReturnBinding, SsoStart, SsoStateContext,
};
use systemprompt_web_admin::{AdfsConfig, AdfsProvider};

const IDP_ENTITY: &str = "https://idp.test/adfs/services/trust";
const IDP_SSO: &str = "https://idp.test/adfs/ls/";
const SP_ENTITY: &str = "https://app.test/saml/metadata";
const ACS: &str = "https://app.test/api/v1/core/sso/adfs/callback";
const EMAIL: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress";
const GROUPS: &str = "http://schemas.xmlsoap.org/claims/Group";

fn idp() -> IdentityProvider {
    let key = KeyPair::from_pkcs8_pem(include_bytes!("../../../fixtures/saml/idp-key.pem"))
        .expect("fixture IdP key")
        .with_certificate(
            X509Certificate::from_pem(include_bytes!("../../../fixtures/saml/idp-cert.pem"))
                .expect("fixture IdP certificate"),
        );
    IdentityProvider::new(IdentityProviderConfig {
        entity_id: IDP_ENTITY.to_owned(),
        sso: vec![Endpoint::redirect(IDP_SSO, 0, true)],
        slo: vec![],
        artifact_resolution: vec![],
        supported_name_id_formats: vec![NameIdFormat::EmailAddress],
        default_name_id_format: NameIdFormat::EmailAddress,
        signing_key: key,
        decryption_key: None,
        want_authn_requests_signed: false,
        assertion_signing: saml::IdpAssertionSigning {
            sign_responses: false,
            sign_assertions: true,
        },
        encrypt_assertions_when_possible: false,
        default_session_duration: Duration::from_secs(3600),
        default_peer_crypto_policy: PeerCryptoPolicy::strong_defaults(),
        outbound_signature_algorithm: SignatureAlgorithm::RsaSha256,
        outbound_digest_algorithm: DigestAlgorithm::Sha256,
        outbound_c14n: C14nAlgorithm::ExclusiveCanonical,
        outbound_data_encryption_algorithm:
            saml::xmlenc::algorithms::DataEncryptionAlgorithm::Aes256Gcm,
        outbound_key_transport_algorithm: saml::xmlenc::algorithms::KeyTransportAlgorithm::RsaOaep,
    })
    .expect("fixture IdP builds")
}

fn config(idp: &IdentityProvider) -> AdfsConfig {
    let mut cfg = AdfsConfig::disabled();
    cfg.enabled = true;
    cfg.entity_id = SP_ENTITY.to_owned();
    cfg.acs_url = ACS.to_owned();
    cfg.idp_metadata_xml = idp.metadata_xml(false).expect("IdP metadata");
    cfg.allowed_email_domains = vec!["example.com".to_owned()];
    cfg.allow_idp_initiated = false;
    cfg
}

struct Signed {
    response: String,
    state: SsoStateContext,
    relay: String,
}

fn query(url: &url::Url, name: &str) -> String {
    url.query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| panic!("{name} rides the redirect"))
}

async fn sign_in(provider: &AdfsProvider, idp: &IdentityProvider) -> Signed {
    let redirect = provider
        .start(SsoStart {
            return_to: "/admin",
            callback_url: ACS,
            reauth: SsoReauth::default(),
        })
        .await
        .expect("the provider starts a sign-in");
    assert!(
        redirect.url.starts_with(IDP_SSO),
        "redirects to the farm: {}",
        redirect.url
    );
    let url = url::Url::parse(&redirect.url).expect("redirect URL");
    let relay = query(&url, "RelayState");
    assert_eq!(
        relay, redirect.seed.state,
        "the RelayState is the remembered state"
    );
    let mut xml = Vec::new();
    DeflateDecoder::new(
        STANDARD
            .decode(query(&url, "SAMLRequest"))
            .expect("base64")
            .as_slice(),
    )
    .read_to_end(&mut xml)
    .expect("inflate the AuthnRequest");

    let metadata = provider
        .metadata()
        .await
        .expect("the provider publishes SP metadata")
        .expect("SP metadata renders");
    let sp = SpDescriptor::from_metadata_xml(metadata.body.as_bytes()).expect("SP metadata parses");
    let now = SystemTime::now();
    let parsed = idp
        .consume_authn_request(ConsumeAuthnRequest {
            sp: &sp,
            peer_crypto_policy: None,
            saml_request: &xml,
            binding: Binding::HttpRedirect,
            relay_state: Some(&relay),
            detached_signature: None,
            expected_destination: IDP_SSO,
            now,
            clock_skew: Duration::from_secs(120),
        })
        .expect("the IdP accepts the AuthnRequest");
    assert_eq!(parsed.assertion_consumer_service.url, ACS);
    let SsoResponseDispatch::Post(form) = idp
        .issue_response(IssueResponse {
            sp: &sp,
            in_response_to: &parsed,
            name_id: NameId::email("ada@example.com"),
            attributes: vec![
                Attribute::single(EMAIL, "Ada@Example.com"),
                Attribute::new(GROUPS, vec!["Systemprompt-Users".to_owned()]),
            ],
            authn_instant: now,
            session_index: "s1".to_owned(),
            session_not_on_or_after: None,
            authn_context_class_ref: AuthnContextClassRef::PasswordProtectedTransport,
            force_encrypt_assertion: Some(false),
            now,
            assertion_lifetime: Duration::from_secs(300),
            subject_confirmation_lifetime: Duration::from_secs(300),
            holder_of_key_cert: None,
        })
        .expect("the IdP signs a response")
    else {
        panic!("a POST response");
    };
    Signed {
        response: form.saml_response,
        state: SsoStateContext {
            request_id: redirect.seed.request_id,
            nonce: redirect.seed.nonce,
            pkce_verifier: None,
        },
        relay,
    }
}

fn returned(response: String, relay: &str) -> SsoReturn {
    SsoReturn {
        binding: SsoReturnBinding::Form,
        params: vec![
            ("SAMLResponse".to_owned(), response),
            ("RelayState".to_owned(), relay.to_owned()),
        ],
    }
}

#[tokio::test]
async fn a_signed_assertion_for_the_request_becomes_the_identity() {
    let idp = idp();
    let provider = AdfsProvider::new(&config(&idp)).expect("an enabled config builds");
    assert_eq!(provider.id(), "adfs");
    assert_eq!(provider.paths().callback, "/api/v1/core/sso/adfs/callback");

    let signed = sign_in(&provider, &idp).await;
    let identity = provider
        .finish(returned(signed.response, &signed.relay), Some(signed.state))
        .await
        .expect("the signed assertion verifies");

    assert_eq!(identity.email.as_deref(), Some("ada@example.com"));
    assert!(identity.email_verified);
    assert_eq!(identity.groups, vec!["Systemprompt-Users".to_owned()]);
    assert!(
        identity.assertion_id.is_some(),
        "the assertion id feeds core's replay guard"
    );
    assert!(identity.authenticated_at.is_some());
}

#[tokio::test]
async fn a_tampered_assertion_is_refused() {
    let idp = idp();
    let provider = AdfsProvider::new(&config(&idp)).expect("an enabled config builds");
    let signed = sign_in(&provider, &idp).await;
    let xml = String::from_utf8(STANDARD.decode(&signed.response).expect("base64")).expect("utf8");
    let forged = STANDARD.encode(xml.replace("Systemprompt-Users", "Systemprompt-Admins"));
    let refused = provider
        .finish(returned(forged, &signed.relay), Some(signed.state))
        .await
        .expect_err("a modified assertion does not verify");
    assert!(
        matches!(refused, SsoError::InvalidAssertion { .. }),
        "{refused}"
    );
}

#[tokio::test]
async fn a_response_to_another_request_is_refused() {
    let idp = idp();
    let provider = AdfsProvider::new(&config(&idp)).expect("an enabled config builds");
    let first = sign_in(&provider, &idp).await;
    let second = sign_in(&provider, &idp).await;
    let refused = provider
        .finish(returned(first.response, &first.relay), Some(second.state))
        .await
        .expect_err("the response answers a different AuthnRequest");
    assert!(
        matches!(refused, SsoError::InvalidAssertion { .. }),
        "{refused}"
    );
}

#[tokio::test]
async fn an_unsolicited_response_is_refused_when_idp_initiated_is_off() {
    let idp = idp();
    let provider = AdfsProvider::new(&config(&idp)).expect("an enabled config builds");
    let signed = sign_in(&provider, &idp).await;
    assert!(
        provider
            .finish(returned(signed.response, &signed.relay), None)
            .await
            .is_err()
    );
}

#[test]
fn an_enabled_config_without_allowed_domains_is_refused() {
    let idp = idp();
    let mut cfg = config(&idp);
    cfg.allowed_email_domains.clear();
    assert!(
        AdfsProvider::new(&cfg).is_err(),
        "no domain is admitted until the operator names one"
    );
}
