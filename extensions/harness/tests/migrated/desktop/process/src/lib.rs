//! Process behaviour of the desktop shell (single instance, links handed over
//! at launch, window placement, preferences persistence, build identity and
//! packaging inputs), migrated case by case from the reference suite: batch 4,
//! first lane. Each test states one behaviour in the harness's own terms; the
//! link from a test to the case it carries lives in `cases.md` beside this
//! crate, never in the test itself. A test marked `#[ignore = "parity ledger:
//! …"]` asserts the reference behaviour where the harness differs.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod files_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod launch_tests;
#[cfg(test)]
mod links_tests;
#[cfg(test)]
mod settings_store_tests;
#[cfg(test)]
mod window_tests;
