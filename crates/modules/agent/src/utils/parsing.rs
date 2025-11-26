use crate::errors::{ArtifactError, ContextError, TaskError};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use systemprompt_core_database::JsonRow;

pub fn required_string_task(row: &JsonRow, field: &str) -> Result<String, TaskError> {
    row.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| TaskError::MissingField {
            field: field.to_string(),
        })
        .map(|s| s.to_string())
}

pub fn required_string_context(row: &JsonRow, field: &str) -> Result<String, ContextError> {
    row.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ContextError::MissingField {
            field: field.to_string(),
        })
        .map(|s| s.to_string())
}

pub fn required_string_artifact(row: &JsonRow, field: &str) -> Result<String, ArtifactError> {
    row.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ArtifactError::MissingField {
            field: field.to_string(),
        })
        .map(|s| s.to_string())
}

pub fn optional_string(row: &JsonRow, field: &str) -> Option<String> {
    row.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn required_datetime_task(row: &JsonRow, field: &str) -> Result<DateTime<Utc>, TaskError> {
    row.get(field)
        .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
        .ok_or_else(|| TaskError::InvalidDatetime {
            field: field.to_string(),
        })
}

pub fn required_datetime_context(
    row: &JsonRow,
    field: &str,
) -> Result<DateTime<Utc>, ContextError> {
    row.get(field)
        .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
        .ok_or_else(|| ContextError::InvalidDatetime {
            field: field.to_string(),
        })
}

pub fn required_datetime_artifact(
    row: &JsonRow,
    field: &str,
) -> Result<DateTime<Utc>, ArtifactError> {
    row.get(field)
        .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
        .ok_or_else(|| ArtifactError::InvalidDatetime {
            field: field.to_string(),
        })
}

pub fn optional_datetime(row: &JsonRow, field: &str) -> Option<DateTime<Utc>> {
    row.get(field)
        .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
}

pub fn optional_json_task<T: DeserializeOwned>(
    row: &JsonRow,
    field: &str,
) -> Result<Option<T>, TaskError> {
    match row.get(field).and_then(|v| v.as_str()) {
        None => Ok(None),
        Some(json_str) if json_str == "null" || json_str.is_empty() => Ok(None),
        Some(json_str) => {
            serde_json::from_str(json_str)
                .map(Some)
                .map_err(|e| TaskError::JsonParse {
                    field: field.to_string(),
                    source: e,
                })
        },
    }
}

pub fn optional_json_context<T: DeserializeOwned>(
    row: &JsonRow,
    field: &str,
) -> Result<Option<T>, ContextError> {
    match row.get(field).and_then(|v| v.as_str()) {
        None => Ok(None),
        Some(json_str) if json_str == "null" || json_str.is_empty() => Ok(None),
        Some(json_str) => {
            serde_json::from_str(json_str)
                .map(Some)
                .map_err(|e| ContextError::JsonParse {
                    field: field.to_string(),
                    source: e,
                })
        },
    }
}

pub fn optional_json_artifact<T: DeserializeOwned>(
    row: &JsonRow,
    field: &str,
) -> Result<Option<T>, ArtifactError> {
    match row.get(field).and_then(|v| v.as_str()) {
        None => Ok(None),
        Some(json_str) if json_str == "null" || json_str.is_empty() => Ok(None),
        Some(json_str) => {
            serde_json::from_str(json_str)
                .map(Some)
                .map_err(|e| ArtifactError::JsonParse {
                    field: field.to_string(),
                    source: e,
                })
        },
    }
}

pub fn required_json_task<T: DeserializeOwned>(row: &JsonRow, field: &str) -> Result<T, TaskError> {
    let json_str =
        row.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::MissingField {
                field: field.to_string(),
            })?;

    serde_json::from_str(json_str).map_err(|e| TaskError::JsonParse {
        field: field.to_string(),
        source: e,
    })
}

pub fn required_json_context<T: DeserializeOwned>(
    row: &JsonRow,
    field: &str,
) -> Result<T, ContextError> {
    let json_str =
        row.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ContextError::MissingField {
                field: field.to_string(),
            })?;

    serde_json::from_str(json_str).map_err(|e| ContextError::JsonParse {
        field: field.to_string(),
        source: e,
    })
}

pub fn required_json_artifact<T: DeserializeOwned>(
    row: &JsonRow,
    field: &str,
) -> Result<T, ArtifactError> {
    let json_str =
        row.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ArtifactError::MissingField {
                field: field.to_string(),
            })?;

    serde_json::from_str(json_str).map_err(|e| ArtifactError::JsonParse {
        field: field.to_string(),
        source: e,
    })
}

pub fn required_bool_task(row: &JsonRow, field: &str) -> Result<bool, TaskError> {
    row.get(field)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| TaskError::MissingField {
            field: field.to_string(),
        })
}

pub fn optional_bool(row: &JsonRow, field: &str) -> Option<bool> {
    row.get(field).and_then(|v| v.as_bool())
}

pub fn required_i64_task(row: &JsonRow, field: &str) -> Result<i64, TaskError> {
    row.get(field)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| TaskError::MissingField {
            field: field.to_string(),
        })
}

pub fn optional_i64(row: &JsonRow, field: &str) -> Option<i64> {
    row.get(field).and_then(|v| v.as_i64())
}

pub fn optional_json_value(row: &JsonRow, field: &str) -> Result<Option<JsonValue>, TaskError> {
    optional_json_task(row, field)
}
