use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Module config missing required field: {field}")]
    MissingConfigField { field: String },

    #[error("Invalid module version: {version}")]
    InvalidVersion { version: String },

    #[error("Module {name} configuration invalid: {reason}")]
    InvalidModuleConfig { name: String, reason: String },

    #[error("Module {name} not found")]
    ModuleNotFound { name: String },

    #[error("Invalid module field {field}: {reason}")]
    InvalidField { field: String, reason: String },

    #[error("Version comparison failed: {reason}")]
    VersionComparisonFailed { reason: String },
}
