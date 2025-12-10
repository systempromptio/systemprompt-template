use super::OAuthRepository;
use anyhow::Result;

const VALID_SCOPES: &[(&str, &str, bool)] = &[
    ("user", "Standard user access", true),
    ("admin", "Administrative access", false),
    ("anonymous", "Anonymous user access", false),
];

impl OAuthRepository {
    pub async fn validate_scopes(&self, requested_scopes: &[String]) -> Result<Vec<String>> {
        if requested_scopes.is_empty() {
            return Ok(vec![]);
        }

        let mut valid_scopes = Vec::new();
        let mut invalid_scopes = Vec::new();

        for scope in requested_scopes {
            if Self::scope_exists_static(scope) {
                valid_scopes.push(scope.clone());
            } else {
                invalid_scopes.push(scope.clone());
            }
        }

        if !invalid_scopes.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid scopes (roles): {}",
                invalid_scopes.join(", ")
            ));
        }

        Ok(valid_scopes)
    }

    pub async fn get_available_scopes(&self) -> Result<Vec<(String, Option<String>)>> {
        Ok(VALID_SCOPES
            .iter()
            .map(|(name, desc, _)| ((*name).to_string(), Some((*desc).to_string())))
            .collect())
    }

    pub async fn scope_exists(&self, scope_name: &str) -> Result<bool> {
        Ok(Self::scope_exists_static(scope_name))
    }

    fn scope_exists_static(scope_name: &str) -> bool {
        VALID_SCOPES.iter().any(|(name, _, _)| *name == scope_name)
    }

    pub fn parse_scopes(scope_string: &str) -> Vec<String> {
        scope_string
            .split_whitespace()
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn format_scopes(scopes: &[String]) -> String {
        scopes.join(" ")
    }

    pub async fn get_default_roles(&self) -> Result<Vec<String>> {
        Ok(VALID_SCOPES
            .iter()
            .filter(|(_, _, is_default)| *is_default)
            .map(|(name, _, _)| (*name).to_string())
            .collect())
    }
}
