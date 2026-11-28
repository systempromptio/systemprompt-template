//! Tests for the Drive screens of systemprompt-harness-views, through
//! the public API.

// What a test or a fixture returns, so a failed step is reported through `?`.
#[cfg(test)]
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod browse_tests;
#[cfg(test)]
mod extension_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod links_tests;
#[cfg(test)]
mod preview_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod strings_tests;
