//! Tests for the views, state and services of systemprompt-harness-views and
//! the launch of systemprompt-harness-shell, through the public API.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// One clock for every id the tests mint, so ids a test compares are ordered.
#[cfg(test)]
static IDS: systemprompt_engine::store::IdClock = systemprompt_engine::store::IdClock::new();

mod app;
mod state;
mod views_panels;
mod views_session;
