//! The one place a test is allowed to decide it cannot run.
//!
//! A suite that returns early when Postgres is missing reports the same green
//! as a suite that ran every assertion, which is how this repository's own
//! contract tier once passed having executed nothing. On a developer machine
//! with no database that trade is worth making; in CI it is precisely the
//! failure the tier exists to catch, so `CI` turns the skip into a panic.

pub fn ci() -> bool {
    std::env::var_os("CI").is_some_and(|v| !v.is_empty() && v != "0" && v != "false")
}

pub fn skip_or_panic(what: &str, reason: &str) -> bool {
    assert!(
        !ci(),
        "missing test prerequisite in CI: {what} -- {reason}. A skipped test in CI is a failed \
         test: provision the prerequisite or delete the test."
    );
    eprintln!("SKIP {what} -- {reason}");
    false
}
