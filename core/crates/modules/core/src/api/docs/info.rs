#[derive(Debug, Copy, Clone)]
pub struct ApiInfo {
    pub title: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

impl Default for ApiInfo {
    fn default() -> Self {
        Self {
            title: "SystemPrompt OS API",
            version: "1.0.0",
            description:
                "SystemPrompt OS is a comprehensive system configuration and management platform.",
        }
    }
}

pub fn build_api_info() -> ApiInfo {
    ApiInfo::default()
}

#[derive(Debug, Copy, Clone)]
pub struct ApiMetadata {
    pub version: &'static str,
    pub build: &'static str,
    pub environment: &'static str,
}

impl Default for ApiMetadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            build: env!("CARGO_PKG_VERSION"),
            environment: "development",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExternalDocs {
    pub description: &'static str,
    pub url: &'static str,
}

impl Default for ExternalDocs {
    fn default() -> Self {
        Self {
            description: "SystemPrompt OS Documentation",
            url: "https://docs.systemprompt.io",
        }
    }
}
