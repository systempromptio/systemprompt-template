//! URL-safe slug generation utilities for agent names

use regex::Regex;

/// Converts an agent name to a URL-safe slug
///
/// Rules:
/// - Convert to lowercase
/// - Replace spaces and special characters with hyphens
/// - Remove consecutive hyphens
/// - Trim leading/trailing hyphens
/// - Ensure uniqueness by appending counter if needed
pub fn generate_slug(name: &str) -> String {
    // Convert to lowercase and replace spaces/special chars with hyphens
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '_' || c == '.' || c == '-' {
                '-'
            } else {
                // Skip other special characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>();

    // Remove consecutive hyphens and trim
    let re = Regex::new(r"-+").unwrap();
    let cleaned = re.replace_all(&slug, "-");

    cleaned.trim_matches('-').to_string()
}

/// Generate a unique slug by checking against existing slugs
pub fn generate_unique_slug(name: &str, existing_slugs: &[String]) -> String {
    let base_slug = generate_slug(name);

    if !existing_slugs.contains(&base_slug) {
        return base_slug;
    }

    // Find a unique variation
    for i in 1..1000 {
        let candidate = format!("{}-{}", base_slug, i);
        if !existing_slugs.contains(&candidate) {
            return candidate;
        }
    }

    // Fallback to UUID if we can't find a unique slug
    format!(
        "{}-{}",
        base_slug,
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_slug_generation() {
        assert_eq!(generate_slug("Simple Test Agent"), "simple-test-agent");
        assert_eq!(generate_slug("My-Cool_Agent.v2"), "my-cool-agent-v2");
        assert_eq!(generate_slug("Agent@#$%^&*()Name"), "agentname");
        assert_eq!(generate_slug("  Spaced  Agent  "), "spaced-agent");
    }

    #[test]
    fn test_unique_slug_generation() {
        let existing = vec!["test-agent".to_string(), "test-agent-1".to_string()];

        assert_eq!(generate_unique_slug("Test Agent", &[]), "test-agent");
        assert_eq!(
            generate_unique_slug("Test Agent", &existing),
            "test-agent-2"
        );
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(generate_slug(""), "");
        assert_eq!(generate_slug("---"), "");
        assert_eq!(generate_slug("123"), "123");
        assert_eq!(generate_slug("Agent-1.0"), "agent-1-0");
    }
}
