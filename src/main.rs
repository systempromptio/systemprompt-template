//! Entry point for the systemprompt.io template binary: runs the composition
//! of the flavour this build selected.

use std::process::ExitCode;

fn main() -> Result<ExitCode, systemprompt::RunError> {
    systemprompt_template::composition().run()
}
