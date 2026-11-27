//! The binding library the phone apps load.
//!
//! One library carries two annotated surfaces: core's `systemprompt-mobile`
//! crate (instances, diagnostics and the one error type) and this crate's own
//! namespace, which is where a fork exports its own Rust to Swift and Kotlin
//! without changing core. The binding generator reads both from the built
//! library and writes one Kotlin package and one Swift module for each.
//!
//! This crate's own surface is [`library_version`]: the version of this
//! binding library, which is the template's release, as distinct from the
//! core version `systemprompt_mobile::diagnostics` reports.
//!
//! On Android the library also exports the one JNI entry point the app calls
//! at start-up (the `android` module).

#[cfg(target_os = "android")]
pub mod android;

#[must_use]
#[uniffi::export]
pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

uniffi::setup_scaffolding!();

// Why: core's crate is linked as an ordinary Rust library, and a linker keeps
// only what something refers to. This refers to its exported functions so the
// one library carries them.
systemprompt_mobile::uniffi_reexport_scaffolding!();
