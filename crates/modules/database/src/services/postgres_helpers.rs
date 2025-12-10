use sqlx::{Column, Row};
use std::collections::HashMap;

use crate::models::{DbValue, ToDbValue};

pub fn row_to_json(row: &sqlx::postgres::PgRow) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    for column in row.columns() {
        let name = column.name().to_string();

        if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::String(v.and_utc().to_rfc3339())
                }),
            );
        } else if let Ok(val) =
            row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column.ordinal())
        {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::String(v.to_rfc3339())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<uuid::Uuid>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::String(v.to_string())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<String>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, serde_json::Value::String),
            );
        } else if let Ok(val) = row.try_get::<Option<i64>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Number(v.into())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<i32>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Number(i64::from(v).into())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<f64>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
            );
        } else if let Ok(val) = row.try_get::<Option<rust_decimal::Decimal>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::json!(v.to_string().parse::<f64>().unwrap_or(0.0))
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<bool>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, serde_json::Value::Bool),
            );
        } else if let Ok(val) = row.try_get::<Option<Vec<String>>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(column.ordinal()) {
            map.insert(name, val.unwrap_or(serde_json::Value::Null));
        } else if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |bytes| {
                    use base64::engine::general_purpose::STANDARD;
                    use base64::Engine;
                    serde_json::Value::String(STANDARD.encode(&bytes))
                }),
            );
        } else {
            map.insert(name, serde_json::Value::Null);
        }
    }

    map
}

pub fn bind_params<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    params: &[&dyn ToDbValue],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for param in params {
        let value = param.to_db_value();
        query = match value {
            DbValue::String(s) => query.bind(s),
            DbValue::Int(i) => query.bind(i),
            DbValue::Float(f) => query.bind(f),
            DbValue::Bool(b) => query.bind(b),
            DbValue::Bytes(b) => query.bind(b),
            DbValue::Timestamp(dt) => query.bind(dt),
            DbValue::StringArray(arr) => query.bind(arr),
            DbValue::NullString => query.bind(None::<String>),
            DbValue::NullInt => query.bind(None::<i64>),
            DbValue::NullFloat => query.bind(None::<f64>),
            DbValue::NullBool => query.bind(None::<bool>),
            DbValue::NullBytes => query.bind(None::<Vec<u8>>),
            DbValue::NullTimestamp => query.bind(None::<chrono::DateTime<chrono::Utc>>),
            DbValue::NullStringArray => query.bind(None::<Vec<String>>),
        };
    }
    query
}
