//! The process-wide rustls crypto provider, installed before any TLS client
//! is built.

// Why: core's reqwest is built with `rustls-no-provider`, so rustls needs a
// process-wide provider before the first client is built or reqwest panics.
// ring is the one core's entry points install too.
pub(crate) fn install_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_some() {
        return;
    }
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        tracing::debug!("rustls crypto provider was installed concurrently");
    }
}
