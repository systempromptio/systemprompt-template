//! sqlx type derivations for common ID types
//!
//! These newtype wrappers enable compile-time type checking and seamless
//! integration with sqlx's `Type` derive macro for direct database binding.

use sqlx::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct UserId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct ContextId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct TraceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct ArtifactId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct ExecutionStepId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct SkillId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct LogId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct ContentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct FileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct ClientId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(transparent)]
pub struct TokenId(pub String);
