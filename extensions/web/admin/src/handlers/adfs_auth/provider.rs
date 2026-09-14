//! The SAML roles built from [`AdfsConfig`]: our service-provider descriptor
//! and the `IdP` descriptor parsed from the pinned federation metadata.
//!
//! Metadata parsing is cached per config so the 70 KB AD FS document is
//! parsed once per process, not once per sign-in.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use saml::{
    DigestAlgorithm, IdpDescriptor, NameIdFormat, PeerCryptoPolicy, ServiceProvider,
    ServiceProviderConfig, SignatureAlgorithm, SpWantSigned, SsoResponseEndpoint,
};
use tokio::sync::RwLock;

use super::AdfsError;
use super::config::AdfsConfig;

static IDP_CACHE: LazyLock<RwLock<HashMap<String, Arc<IdpDescriptor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub(super) fn service_provider(cfg: &AdfsConfig) -> Result<ServiceProvider, AdfsError> {
    ServiceProvider::new(ServiceProviderConfig {
        entity_id: cfg.entity_id.clone(),
        acs: vec![SsoResponseEndpoint::post(cfg.acs_url.clone(), 0, true)],
        slo: vec![],
        name_id_formats: vec![NameIdFormat::EmailAddress, NameIdFormat::Persistent],
        signing_key: None,
        decryption_key: None,
        sign_authn_requests: false,
        // Why: the assertion is what carries the claims, so the assertion must
        // be signed; AD FS signs it by default and does not sign the outer
        // Response unless asked.
        want_signed: SpWantSigned {
            response: false,
            assertions: true,
        },
        allow_unsolicited: cfg.allow_idp_initiated,
        default_peer_crypto_policy: PeerCryptoPolicy::strong_defaults(),
        outbound_signature_algorithm: SignatureAlgorithm::RsaSha256,
        outbound_digest_algorithm: DigestAlgorithm::Sha256,
    })
    .map_err(|e| AdfsError::ServiceProvider(e.to_string()))
}

// Why: keyed on the metadata text itself, so a re-vendored metadata file is
// a new cache entry rather than a stale descriptor.
pub(super) async fn idp_descriptor(cfg: &AdfsConfig) -> Result<Arc<IdpDescriptor>, AdfsError> {
    if let Some(idp) = IDP_CACHE.read().await.get(&cfg.idp_metadata_xml) {
        return Ok(Arc::clone(idp));
    }
    let parsed = IdpDescriptor::from_metadata_xml(cfg.idp_metadata_xml.as_bytes())
        .map_err(|e| AdfsError::Metadata(e.to_string()))?;
    let idp = Arc::new(parsed);
    IDP_CACHE
        .write()
        .await
        .insert(cfg.idp_metadata_xml.clone(), Arc::clone(&idp));
    Ok(idp)
}
