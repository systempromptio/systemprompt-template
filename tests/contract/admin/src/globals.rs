//! Process-wide globals the admin handlers read, installed once per test
//! binary.
//!
//! `Config`, `ProfileBootstrap`, the secrets bundle, and the JWT signing
//! authority are all `OnceLock`s in core, and `cargo test` runs every test in
//! one process against one set of them. Installing them behind a `Once` keeps
//! the whole binary agreed on a single issuer, which is what makes a token
//! minted in [`crate::principal`] validate inside the handler.
//!
//! The profile is a checked-in fixture rather than the developer's
//! `.systemprompt` profile, which is git-ignored. Depending on the latter
//! would make the suite pass locally and silently skip in CI — the one place
//! the gate has to bite — and would couple the assertions to whatever a given
//! machine happens to have configured.

use std::path::{Path, PathBuf};
use std::sync::{Once, OnceLock};

static INIT: Once = Once::new();
static READY: OnceLock<bool> = OnceLock::new();

// Any absolute URL works: it only has to be a well-formed issuer that both
// the minter and the validator agree on.
const ISSUER: &str = "http://localhost:8099";

// Absolute path to the repository root, derived from this crate's manifest
// directory (`tests/contract/admin`).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("crate sits three levels below the repository root")
        .to_path_buf()
}

// Install the profile, secrets, config, and an ephemeral signing key.
//
// Returns `false` only if the fixture cannot be materialised; the suite then
// self-skips the same way it does without a database.
pub fn init() -> bool {
    INIT.call_once(|| {
        let ready = try_init();
        let _ = READY.set(ready);
    });
    *READY.get().unwrap_or(&false)
}

fn try_init() -> bool {
    let Some(profile) = write_fixture_profile() else {
        return false;
    };

    systemprompt::config::ProfileBootstrap::init_from_path(&profile)
        .expect("initialise the contract fixture profile");
    systemprompt::config::SecretsBootstrap::try_init().expect("load the fixture profile's secrets");
    systemprompt::config::try_init_config().expect("build config from the fixture profile");

    // The local profile ships no signing key. A key generated per process is
    // enough because the same authority both mints and validates here.
    let key = systemprompt_security::keys::RsaSigningKey::generate()
        .expect("generate an ephemeral RSA signing key");
    systemprompt_security::keys::authority::install_for_test(key);
    true
}

// The checked-in profile, with `__REPO__` resolved to this checkout.
//
// Deliberately a fixture rather than the developer's `.systemprompt` profile,
// which is git-ignored: relying on it would make the suite pass locally and
// silently skip in CI, which is the one place the gate has to bite.
const FIXTURE_PROFILE: &str = include_str!("../fixtures/profile.yaml");

// Minimal secrets bundle. Nothing here is a real credential — the suite talks
// to no provider, and the signing key is generated in-process.
//
// `database_url` is required by the schema but unused: the harness opens its
// own throwaway database in `crate::tempdb` and hands that pool to the
// router, so it is echoed back rather than pointed anywhere live.
fn fixture_secrets() -> String {
    let database_url = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/postgres".to_owned());
    format!(
        r#"{{
  "database_url": "{database_url}",
  "oauth_at_rest_pepper": "contract-suite-pepper-not-a-real-secret",
  "manifest_signing_secret_seed": "Y29udHJhY3Qtc3VpdGUtc2VlZC1ub3QtcmVhbC0wMDA="
}}"#
    )
}

// Materialise the fixture profile under `tests/target/`, which is already
// ignored. Secrets live beside the profile because `secrets_path` is relative
// to it.
fn write_fixture_profile() -> Option<PathBuf> {
    let root = repo_root();
    let dir = root.join("tests/target/contract-profile");
    std::fs::create_dir_all(&dir).expect("create fixture profile directory");

    let yaml = FIXTURE_PROFILE
        .replace("__REPO__", &root.to_string_lossy())
        .replace("jwt_issuer: http://localhost:8099", &format!("jwt_issuer: {ISSUER}"));
    std::fs::write(dir.join("profile.yaml"), yaml).expect("write fixture profile");
    std::fs::write(dir.join("secrets.json"), fixture_secrets()).expect("write fixture secrets");

    Some(dir.join("profile.yaml"))
}

// The issuer the admin cookie validator will check tokens against.
pub fn jwt_issuer() -> String {
    systemprompt::models::Config::get()
        .expect("config installed by init()")
        .jwt_issuer
        .clone()
}
