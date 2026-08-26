//! `systemprompt plugins run platform` — platform-organization membership.
//!
//! Process shell only: install logging, start a runtime, hand the parsed
//! arguments to the library. The parse surface and the subcommands live in
//! [`systemprompt_cli_platform`].

// Why: stderr is how this binary reports a failure before the library's
// printing takes over; the workspace lints deny printing by default.
#![allow(
    clippy::print_stderr,
    reason = "CLI binary: stderr is the user-facing error channel"
)]

use std::process::ExitCode;

use clap::Parser;
use systemprompt_cli_platform::cli::Cli;
use systemprompt_cli_platform::commands;

fn main() -> ExitCode {
    systemprompt::logging::init_console_logging();

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: could not start async runtime: {e}");
            return ExitCode::FAILURE;
        },
    };

    match runtime.block_on(commands::run(Cli::parse())) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        },
    }
}
