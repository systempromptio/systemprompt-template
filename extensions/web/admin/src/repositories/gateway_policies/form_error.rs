//! What the policy editor refuses, in the operator's words.
//!
//! One variant per refusal so the handler never composes a message; the
//! numeric variants keep the parse error that said so.

/// What was wrong with the form, in the operator's words; the numeric
/// variants keep the parse error that said so.
#[derive(Debug, thiserror::Error)]
pub enum FormError {
    #[error("a policy needs a name")]
    MissingName,
    #[error("policy names are letters, digits, '-' and '_'")]
    BadName,
    #[error("{field} must be a whole number: {source}")]
    NotWhole {
        field: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("{field} must be an amount in dollars: {source}")]
    NotDollars {
        field: String,
        #[source]
        source: std::num::ParseFloatError,
    },
    #[error("{field} must not be negative")]
    Negative { field: String },
    #[error("window {index} has no length")]
    NoLength { index: usize },
    #[error("window {index}: '{subject}' is not a quota subject")]
    UnknownSubject { index: usize, subject: String },
    #[error("window {index} sets no ceiling")]
    NoCeiling { index: usize },
    #[error("{field}: '{value}' is neither warn nor enforce")]
    BadMode { field: String, value: String },
    #[error("history: '{value}' is not off, audit or block")]
    BadHistory { value: String },
    #[error("'{scanner}' is not a scanner on this instance")]
    UnknownScanner { scanner: String },
    #[error("{0}")]
    Invalid(#[source] systemprompt::gateway::GatewayPolicyError),
}
