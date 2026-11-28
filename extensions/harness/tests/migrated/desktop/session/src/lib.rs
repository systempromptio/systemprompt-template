//! The session surface of the desktop shell (the transcript model and the rows
//! it becomes, tool rows, streamed markdown blocks, the session list, the
//! composer's lists and the request docks), migrated case by case from the
//! reference suite: batch 4, second lane. Each test states one behaviour in the
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
mod docks_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod markdown_tests;
#[cfg(test)]
mod session_list_tests;
#[cfg(test)]
mod tool_rows_tests;
#[cfg(test)]
mod transcript_tests;
