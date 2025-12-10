use super::ProtocolHandler;

#[derive(Debug, Clone, Copy)]
pub struct A2aProtocol;

impl ProtocolHandler for A2aProtocol {
    fn protocol_name(&self) -> &'static str {
        "a2a"
    }

    fn get_base_path(&self) -> &'static str {
        "/"
    }
}

impl Default for A2aProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl A2aProtocol {
    pub const fn new() -> Self {
        Self
    }
}
