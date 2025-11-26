#[derive(Debug, Copy, Clone)]
pub enum SecurityRequirement {
    None,
    BearerAuth,
    ApiKey,
    Either,
}

impl SecurityRequirement {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BearerAuth => "bearer_auth",
            Self::ApiKey => "api_key",
            Self::Either => "either",
        }
    }
}
