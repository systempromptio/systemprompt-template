//! Tests for the shell extension contract of systemprompt-harness-shared,
//! through the public API.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod discovery_tests;
