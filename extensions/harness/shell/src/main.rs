#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! The `harness` executable: the shell's entry point, everything else lives in
//! the `harness_desktop` library.

use std::process::ExitCode;

fn main() -> ExitCode {
    harness_desktop::run()
}
