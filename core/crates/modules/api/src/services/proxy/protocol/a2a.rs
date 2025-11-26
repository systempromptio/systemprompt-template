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

impl A2aProtocol {
    pub fn new() -> Self {
        Self
    }
}
