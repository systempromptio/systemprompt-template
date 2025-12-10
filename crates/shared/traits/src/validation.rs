use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub context: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            context: None,
        }
    }

    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref ctx) = self.context {
            write!(
                f,
                "VALIDATION ERROR [{}]: {} (context: {})",
                self.field, self.message, ctx
            )
        } else {
            write!(f, "VALIDATION ERROR [{}]: {}", self.field, self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub trait Validate: Debug {
    fn validate(&self) -> ValidationResult<()>;
}

pub trait MetadataValidation: Validate {
    fn required_string_fields(&self) -> Vec<(&'static str, &str)>;

    fn validate_required_fields(&self) -> ValidationResult<()> {
        for (field_name, field_value) in self.required_string_fields() {
            if field_value.is_empty() {
                return Err(ValidationError::new(
                    field_name,
                    format!("{field_name} cannot be empty"),
                ));
            }
        }
        Ok(())
    }
}
