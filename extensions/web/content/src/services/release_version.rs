//! Release-version substitution for content bodies.
//!
//! Hosted documentation must never name a version the running crates are not
//! on. Pages write `@RELEASE_VERSION@` and ingestion replaces it with this
//! crate's version, which the workspace inherits from the release pin, so a
//! version bump re-renders every page that mentions it and nothing else can
//! drift. The hash stored per file is computed after substitution, so a bump
//! alone is enough to re-ingest.

pub const RELEASE_VERSION_TOKEN: &str = "@RELEASE_VERSION@";

pub const RELEASE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn substitute_release_version(text: &str) -> String {
    substitute_with(text, RELEASE_VERSION)
}

#[must_use]
pub fn substitute_with(text: &str, version: &str) -> String {
    if text.contains(RELEASE_VERSION_TOKEN) {
        text.replace(RELEASE_VERSION_TOKEN, version)
    } else {
        text.to_owned()
    }
}
