//! Entry point for the systemprompt.io template binary.
//!
//! The binary runs the template's composition, which hosts the `systemprompt`
//! MCP toolset as `admin mcp serve --server systemprompt`.

use std::process::ExitCode;

fn main() -> Result<ExitCode, systemprompt::RunError> {
    systemprompt_template::composition().run()
}
