use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ExtensionConfigErrors {
    pub extension: &'static str,
    pub errors: Vec<ExtensionConfigError>,
}

#[derive(Debug)]
pub struct ExtensionConfigError {
    pub field: String,
    pub message: String,
    pub path: Option<PathBuf>,
    pub suggestion: Option<String>,
}

impl ExtensionConfigErrors {
    #[must_use]
    pub fn new(extension: &'static str) -> Self {
        Self {
            extension,
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: None,
            suggestion: None,
        });
    }

    pub fn push_with_path(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<PathBuf>,
    ) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: Some(path.into()),
            suggestion: None,
        });
    }

    pub fn push_with_suggestion(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: None,
            suggestion: Some(suggestion.into()),
        });
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for ExtensionConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Extension '{}' configuration errors:", self.extension)?;
        for error in &self.errors {
            write!(f, "  [{}] {}", error.field, error.message)?;
            if let Some(path) = &error.path {
                write!(f, "\n    Path: {}", path.display())?;
            }
            if let Some(suggestion) = &error.suggestion {
                write!(f, "\n    Fix: {suggestion}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for ExtensionConfigErrors {}
