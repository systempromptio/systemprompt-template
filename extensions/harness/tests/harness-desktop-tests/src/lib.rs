//! Tests for the views, state and services of systemprompt-harness-views and
//! the launch of systemprompt-harness-shell, through the public API.

// One clock for every id the tests mint, so ids a test compares are ordered.
#[cfg(test)]
static IDS: systemprompt_engine::store::IdClock = systemprompt_engine::store::IdClock::new();

mod app;
mod state;
mod views_panels;
mod views_session;
