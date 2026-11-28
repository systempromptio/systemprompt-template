//! Tests for the notifications and Slack screens of systemprompt-harness-views,
//! through the public API.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod extension_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod strings_tests;
#[cfg(test)]
mod support;
