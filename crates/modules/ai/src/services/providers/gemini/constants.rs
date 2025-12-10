use std::time::Duration;

pub mod timeout {
    use super::Duration;

    pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
}

pub mod tokens {
    pub const DEFAULT_MAX_OUTPUT: u32 = 4096;
    pub const EXTENDED_MAX_OUTPUT: u32 = 8192;
    pub const THINKING_BUDGET: u32 = 8192;
}

pub mod defaults {
    pub const RELEVANCE_SCORE: f32 = 0.85;
    pub const ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta";
}
