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

impl McpProtocol {
    pub fn new() -> Self {
        Self
    }
}
