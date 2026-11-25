//! Entry point for the systemprompt.io template binary.
//!
//! The binary is the reference composition with the template's members added
//! with install standing: the web extension and the `systemprompt` MCP
//! toolset, which the binary hosts as `admin mcp serve --server systemprompt`.

use std::process::ExitCode;

use systemprompt_mcp_agent::SystempromptMcpExtension;
use systemprompt_web_extension::WebExtension;

fn main() -> Result<ExitCode, systemprompt::RunError> {
    systemprompt::reference()
        .with(WebExtension)
        .with(SystempromptMcpExtension)
        .run()
}
