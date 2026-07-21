//! Encrypted secret storage.
//!
//! Secrets are sealed with a per-user data encryption key, which is itself
//! sealed under the instance master key. Plaintext never reaches the database.

pub mod secret_audit;
pub mod secret_crypto;
pub mod secret_keys;
pub mod secret_migration;
pub mod secret_resolve;
