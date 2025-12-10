use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

/// Type alias for database rows represented as JSON objects.
///
/// Each row is a `HashMap` where keys are column names and values are JSON
/// values.
pub type JsonRow = HashMap<String, serde_json::Value>;

/// Parse datetime from `PostgreSQL` database value.
///
/// `PostgreSQL` stores datetimes in multiple formats:
/// - **`PostgreSQL` TIMESTAMP**: "YYYY-MM-DD HH:MM:SS.ffffff"
///   (`CURRENT_TIMESTAMP` with fractional seconds)
/// - **RFC3339**: "2025-01-01T12:00:00+00:00" (used by programmatic inserts)
/// - **Unix timestamp**: Integer (seconds since epoch)
///
/// This helper handles all formats for consistent datetime parsing.
#[must_use]
pub fn parse_database_datetime(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    if let Some(s) = value.as_str() {
        // Try PostgreSQL TIMESTAMP format (with fractional seconds)
        // Format: "2025-01-01 12:00:00.123456"
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
            return Some(dt.and_utc());
        }

        // Try RFC3339 format (used by programmatic inserts)
        // Format: "2025-01-01T12:00:00+00:00"
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }

        None
    } else if let Some(ts) = value.as_i64() {
        DateTime::from_timestamp(ts, 0)
    } else {
        None
    }
}

/// Database value enum representing all possible `PostgreSQL` column types.
///
/// This enum provides a unified representation of `PostgreSQL` database values.
/// Type-specific NULL variants ensure proper `PostgreSQL` type coercion when
/// binding parameters.
#[derive(Debug, Clone)]
pub enum DbValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Timestamp(DateTime<Utc>),
    StringArray(Vec<String>),
    NullString,
    NullInt,
    NullFloat,
    NullBool,
    NullBytes,
    NullTimestamp,
    NullStringArray,
}

/// Trait for converting Rust types to database-compatible values.
///
/// Implement this trait for custom types that need to be used as query
/// parameters.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{DbValue, ToDbValue};
///
/// struct UserId(String);
///
/// impl ToDbValue for UserId {
///     fn to_db_value(&self) -> DbValue {
///         DbValue::String(self.0.clone())
///     }
/// }
/// ```
pub trait ToDbValue: Send + Sync {
    fn to_db_value(&self) -> DbValue;

    /// Returns the appropriate NULL variant for this type.
    /// Used by Option<T> to return type-specific NULLs.
    #[must_use]
    fn null_db_value() -> DbValue
    where
        Self: Sized,
    {
        DbValue::NullString
    }
}

impl ToDbValue for &str {
    fn to_db_value(&self) -> DbValue {
        DbValue::String((*self).to_string())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for String {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for &String {
    fn to_db_value(&self) -> DbValue {
        DbValue::String((*self).clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for i32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for i64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for u32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for u64 {
    #[allow(clippy::cast_possible_wrap)]
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(*self as i64)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for f32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(f64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for f64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for &f64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for &i32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(**self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for &i64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for &bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bool(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBool
    }
}

impl ToDbValue for bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bool(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBool
    }
}

impl ToDbValue for Vec<u8> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bytes(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBytes
    }
}

impl ToDbValue for &[u8] {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bytes(self.to_vec())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBytes
    }
}

impl<T: ToDbValue> ToDbValue for Option<T> {
    fn to_db_value(&self) -> DbValue {
        self.as_ref()
            .map_or_else(T::null_db_value, ToDbValue::to_db_value)
    }
}

impl ToDbValue for DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Timestamp(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullTimestamp
    }
}

impl ToDbValue for &DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Timestamp(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullTimestamp
    }
}

impl ToDbValue for Vec<String> {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

impl ToDbValue for &Vec<String> {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray((*self).clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

impl ToDbValue for &[String] {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray(self.to_vec())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

/// Trait for converting database values to Rust types.
///
/// Implement this trait for custom types that need to be deserialized from
/// query results.
///
/// # Example
///
/// ```rust
/// use anyhow::{anyhow, Result};
/// use systemprompt_database::{DbValue, FromDbValue};
///
/// struct UserId(String);
///
/// impl FromDbValue for UserId {
///     fn from_db_value(value: &DbValue) -> Result<Self> {
///         match value {
///             DbValue::String(s) => Ok(UserId(s.clone())),
///             _ => Err(anyhow!("Invalid UserId type")),
///         }
///     }
/// }
/// ```
pub trait FromDbValue: Sized {
    fn from_db_value(value: &DbValue) -> Result<Self>;
}

impl FromDbValue for String {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::String(s) => Ok(s.clone()),
            DbValue::Int(i) => Ok(i.to_string()),
            DbValue::Float(f) => Ok(f.to_string()),
            DbValue::Bool(b) => Ok(b.to_string()),
            DbValue::Timestamp(dt) => Ok(dt.to_rfc3339()),
            DbValue::StringArray(arr) => {
                Ok(serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string()))
            },
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to String")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to String")),
        }
    }
}

impl FromDbValue for i64 {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Int(i) => Ok(*i),
            #[allow(clippy::cast_possible_truncation)]
            DbValue::Float(f) => Ok(*f as Self),
            DbValue::Bool(b) => Ok(Self::from(*b)),
            DbValue::String(s) => s.parse().map_err(|_| anyhow!("Cannot parse {s} as i64")),
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to i64")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to i64")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to i64")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to i64")),
        }
    }
}

impl FromDbValue for i32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for u64 {
    #[allow(clippy::cast_sign_loss)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for f64 {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Float(f) => Ok(*f),
            #[allow(clippy::cast_precision_loss)]
            DbValue::Int(i) => Ok(*i as Self),
            DbValue::String(s) => s.parse().map_err(|_| anyhow!("Cannot parse {s} as f64")),
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to f64")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to f64")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to f64")),
            DbValue::Bool(_) => Err(anyhow!("Cannot convert Bool to f64")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to f64")),
        }
    }
}

impl FromDbValue for bool {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Bool(b) => Ok(*b),
            DbValue::Int(i) => Ok(*i != 0),
            DbValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(true),
                "false" | "0" | "no" => Ok(false),
                _ => Err(anyhow!("Cannot parse {s} as bool")),
            },
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to bool")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to bool")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to bool")),
            DbValue::Float(_) => Err(anyhow!("Cannot convert Float to bool")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to bool")),
        }
    }
}

impl FromDbValue for Vec<u8> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Bytes(b) => Ok(b.clone()),
            DbValue::String(s) => Ok(s.as_bytes().to_vec()),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to Vec<u8>")),
            _ => Err(anyhow!("Cannot convert {value:?} to Vec<u8>")),
        }
    }
}

impl<T: FromDbValue> FromDbValue for Option<T> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Ok(None),
            _ => T::from_db_value(value).map(Some),
        }
    }
}

impl FromDbValue for DateTime<Utc> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::String(s) => parse_database_datetime(&serde_json::Value::String(s.clone()))
                .ok_or_else(|| anyhow!("Cannot parse {s} as DateTime<Utc>")),
            DbValue::Timestamp(dt) => Ok(*dt),
            DbValue::Int(ts) => {
                Self::from_timestamp(*ts, 0).ok_or_else(|| anyhow!("Invalid Unix timestamp: {ts}"))
            },
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to DateTime<Utc>")),
            _ => Err(anyhow!("Cannot convert {value:?} to DateTime<Utc>")),
        }
    }
}

/// `PostgreSQL` database query.
///
/// This struct holds a `PostgreSQL` query that can be executed through the
/// [`DatabaseProvider`]. Stores a `PostgreSQL` query that is loaded at compile
/// time.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::DatabaseQuery;
///
/// const CREATE_USER: DatabaseQuery =
///     DatabaseQuery::new(include_str!("queries/postgres/create_user.sql"));
///
/// db.execute(&CREATE_USER, &[&"Alice"]).await?;
/// ```
///
/// Use the `database_query!` macro for cleaner syntax:
///
/// ```rust
/// const CREATE_USER: DatabaseQuery = database_query!("users/create");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DatabaseQuery {
    postgres: &'static str,
}

impl DatabaseQuery {
    #[must_use]
    pub const fn new(query: &'static str) -> Self {
        Self { postgres: query }
    }

    /// Get the `PostgreSQL` query string (only variant supported)
    #[must_use]
    pub const fn postgres(&self) -> &str {
        self.postgres
    }

    /// Deprecated: Use `postgres()` instead. Kept for backward compatibility.
    #[deprecated(
        since = "0.1.0",
        note = "Use postgres() instead - SQLite is no longer supported"
    )]
    #[must_use]
    pub const fn select(&self, _is_postgres: bool) -> &str {
        self.postgres
    }

    /// Deprecated: Use `postgres()` instead. Kept for backward compatibility.
    #[deprecated(
        since = "0.1.0",
        note = "Use postgres() instead - SQLite is no longer supported"
    )]
    pub fn get(&self, _db: &dyn crate::DatabaseProvider) -> &str {
        self.postgres
    }
}
