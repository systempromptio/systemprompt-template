#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! The `harness` executable: the shell's entry point, everything else lives in
//! the `systemprompt_harness_shell` library.

use std::process::ExitCode;

fn main() -> ExitCode {
    systemprompt_harness_shell::run()
}
