pub mod a2a;
pub mod mcp;

pub trait ProtocolHandler {
    fn protocol_name(&self) -> &'static str;
    fn get_base_path(&self) -> &'static str;
}

pub use a2a::A2aProtocol;
pub use mcp::McpProtocol;
