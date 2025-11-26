use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRequirement {
    None,
    UserOnly,
    UserWithContext,
    McpWithHeaders,
}

impl fmt::Display for ContextRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::UserOnly => write!(f, "user-only"),
            Self::UserWithContext => write!(f, "user-with-context"),
            Self::McpWithHeaders => write!(f, "mcp-with-headers"),
        }
    }
}

impl Default for ContextRequirement {
    fn default() -> Self {
        Self::UserWithContext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(ContextRequirement::None.to_string(), "none");
        assert_eq!(ContextRequirement::UserOnly.to_string(), "user-only");
        assert_eq!(
            ContextRequirement::UserWithContext.to_string(),
            "user-with-context"
        );
    }

    #[test]
    fn test_default_is_most_restrictive() {
        assert_eq!(
            ContextRequirement::default(),
            ContextRequirement::UserWithContext
        );
    }
}
