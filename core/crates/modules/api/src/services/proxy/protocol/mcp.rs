use super::ProtocolHandler;

#[derive(Debug, Clone, Copy)]
pub struct McpProtocol;

impl ProtocolHandler for McpProtocol {
    fn protocol_name(&self) -> &'static str {
        "mcp"
    }

    fn get_base_path(&self) -> &'static str {
        "/mcp"
    }
}

impl Default for McpProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl McpProtocol {
    pub const fn new() -> Self {
        Self
    }
}
