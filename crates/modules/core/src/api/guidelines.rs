/// API Schema Naming Standards
///
/// This module defines the comprehensive naming conventions for all API schemas
/// across the `SystemPrompt` OS codebase to ensure consistency and clarity.
///
/// ## REQUEST SCHEMAS (Input from client):
/// - **Pattern**: `{Action}{Resource}Request`
/// - **Examples**: `CreateUserRequest`, `UpdateVariableRequest`,
///   `DeleteClientRequest`
/// - **Use**: HTTP request bodies, form data
/// - **Rule**: Always end with "Request" suffix for clarity about data flow
///   direction
///
/// ## RESPONSE SCHEMAS (Output to client):
/// - **Pattern**: `{Resource}Response` or just `{Resource}` (for simple cases)
/// - **Examples**: `UserResponse`, `VariableResponse`, `TokenResponse`
/// - **Use**: HTTP response bodies, single resources
/// - **Rule**: Prefer "Response" suffix for consistency, except for
///   well-established patterns
///
/// ## QUERY SCHEMAS (URL parameters):
/// - **Pattern**: `{Context}Query` or `{Action}Params`
/// - **Examples**: `ListUsersQuery`, `SearchVariablesParams`
/// - **Use**: Query string parameters, filters, pagination
/// - **Rule**: Use "Query" for read operations, "Params" for parameterized
///   actions
///
/// ## ERROR SCHEMAS:
/// - **Pattern**: `{Context}Error` or `{Resource}Error`
/// - **Examples**: `ValidationError`, `AuthenticationError`, `OAuthError`
/// - **Use**: Error responses, validation failures
/// - **Rule**: Always descriptive of the error context
///
/// ## AVOID:
/// - ❌ "Dto" suffix (outdated Java convention)
/// - ❌ Generic names without context (`Request`, `Response`, `Data`)
/// - ❌ Auto-generated compound names (`CollectionResponse_User`)
/// - ❌ Inconsistent casing or separators
///
/// ## SERDE COMPATIBILITY:
/// - Use `#[serde(rename_all = "snake_case")]` to maintain JSON compatibility
/// - Struct names are for Rust code clarity, JSON field names follow
///   `snake_case`
/// - Schema renames should not affect API contract serialization
///
/// ## EXAMPLES:
/// ```rust
/// // ✅ GOOD - Clear request schema
/// #[derive(Deserialize)]
/// #[serde(rename_all = "snake_case")]
/// pub struct CreateUserRequest {
///     pub name: String,
///     pub email: String,
/// }
///
/// // ✅ GOOD - Clear response schema
/// #[derive(Serialize)]
/// #[serde(rename_all = "snake_case")]
/// pub struct UserResponse {
///     pub uuid: String,
///     pub name: String,
///     pub email: String,
/// }
///
/// // ❌ BAD - Outdated naming
/// pub struct UserDto { ... }
///
/// // ❌ BAD - Unclear purpose
/// pub struct UserData { ... }
/// ```
///
/// ## PARAMETER NAMING STANDARDS
///
/// All API parameters MUST use `snake_case`:
/// - **Query parameters**: ?`per_page=20&client_id=abc`
/// - **JSON request bodies**: {"`client_id"`: "abc", "`redirect_uri"`: "..."}
/// - **Path parameters**: /`users/{user_id`}
/// - **Header names**: X-Request-Id (except standard HTTP headers)
///
/// ### Examples:
/// - ✅ `per_page`, `client_id`, `redirect_uri`, `created_at`, `user_id`
/// - ❌ perPage, clientId, redirect-uri, createdAt, userId
///
/// ### Rationale:
/// - **REST API Standards**: `snake_case` is the widely accepted standard for
///   REST APIs
/// - **Consistency**: Uniform naming reduces cognitive load for developers
/// - **JSON Serialization**: Works naturally with serde's default behavior
/// - **Database Mapping**: Aligns with SQL column naming conventions
///
/// ### Implementation:
/// ```rust
/// #[derive(Serialize, Deserialize)]
/// #[serde(rename_all = "snake_case")] // Ensures JSON uses snake_case
/// pub struct CreateOAuthClientRequest {
///     pub client_id: String,         // ✅ Correct - snake_case field
///     pub redirect_uri: String,      // ✅ Correct - snake_case field
///     pub created_at: DateTime<Utc>, // ✅ Correct - snake_case field
/// }
///
/// #[derive(Deserialize)]
/// pub struct ListQuery {
///     pub per_page: Option<u32>,    // ✅ Correct - snake_case query param
///     pub page_number: Option<u32>, // ✅ Correct - snake_case query param
///     pub sort_by: Option<String>,  // ✅ Correct - snake_case query param
/// }
/// ```
///
/// ### Validation:
/// Use the validation script to check compliance:
/// ```bash
/// ./scripts/validate_api_casing.sh
/// ```
#[derive(Debug, Copy, Clone)]
pub struct ApiNamingGuidelines;

impl ApiNamingGuidelines {
    /// Validates that a field name follows `snake_case` convention
    pub fn is_valid_snake_case(field_name: &str) -> bool {
        if field_name.is_empty() {
            return false;
        }

        if let Some(first_char) = field_name.chars().next() {
            if !first_char.is_ascii_lowercase() && first_char != '_' {
                return false;
            }
        } else {
            return false;
        }

        field_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
}
