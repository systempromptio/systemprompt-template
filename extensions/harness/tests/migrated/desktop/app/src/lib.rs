//! Application views of the desktop shell (the session list, the composer's
//! completion lists and submissions, the pickers, links into the app, relative
//! times and the saved preferences), migrated case by case from the reference
//! suite: batch 4, the application lane. Each test states one behaviour in the
//! harness's own terms; the link from a test to the case it carries lives in
//! `cases.md` beside this crate, never in the test itself. A test marked
//! `#[ignore = "parity ledger: …"]` asserts the reference behaviour where the
//! harness differs.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// One clock for every id the tests mint, so ids a test compares are ordered.
#[cfg(test)]
static IDS: systemprompt_engine::store::IdClock = systemprompt_engine::store::IdClock::new();

#[cfg(test)]
mod links_tests;
#[cfg(test)]
mod pickers_tests;
#[cfg(test)]
mod popup_tests;
#[cfg(test)]
mod preferences_tests;
#[cfg(test)]
mod session_list_tests;
#[cfg(test)]
mod submission_tests;
#[cfg(test)]
mod time_tests;
