//! Evaluation domain failures retain their HTTP classification.

use super::AdminError;

impl From<systemprompt::system::optimization::OptimizationError> for AdminError {
    fn from(error: systemprompt::system::optimization::OptimizationError) -> Self {
        use systemprompt::system::optimization::OptimizationError;
        match error {
            OptimizationError::Evaluation(error) => error.into(),
            OptimizationError::Managed(error) => error.into(),
            OptimizationError::Source(message) => Self::Conflict(message),
            OptimizationError::Json(error) => Self::internal(error),
        }
    }
}

impl From<systemprompt::evaluation::EvaluationError> for AdminError {
    fn from(error: systemprompt::evaluation::EvaluationError) -> Self {
        use systemprompt::evaluation::EvaluationError;
        match error {
            EvaluationError::ResourceNotFound(message) => Self::NotFound(message),
            EvaluationError::ExperimentConflict(message) => Self::Conflict(message),
            EvaluationError::InvalidSpec(message) => Self::Unprocessable(message),
            EvaluationError::BudgetExhausted { .. } => Self::Conflict(error.to_string()),
            EvaluationError::RunNotFound(message) | EvaluationError::RubricNotFound(message) => {
                Self::NotFound(message)
            },
            other => Self::internal(other),
        }
    }
}
