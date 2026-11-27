#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! The `harness` executable: the shell's entry point, everything else lives in
//! the `harness_desktop` library.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::process::ExitCode;

fn main() -> ExitCode {
    harness_desktop::run()
}
