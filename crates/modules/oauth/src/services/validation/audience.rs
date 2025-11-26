use systemprompt_models::auth::JwtAudience;

pub fn validate_service_access(token_audiences: &[JwtAudience], _service_name: &str) -> bool {
    const GLOBAL: &[JwtAudience] = &[
        JwtAudience::Api,
        JwtAudience::Mcp,
        JwtAudience::A2a,
        JwtAudience::Web,
    ];
    GLOBAL.iter().any(|aud| token_audiences.contains(aud))
}

pub fn validate_required_audience(token_audiences: &[JwtAudience], required: JwtAudience) -> bool {
    token_audiences.contains(&required)
}

pub fn validate_any_audience(token_audiences: &[JwtAudience], allowed: &[JwtAudience]) -> bool {
    allowed.iter().any(|aud| token_audiences.contains(aud))
}
