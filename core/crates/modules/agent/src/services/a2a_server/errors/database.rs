use systemprompt_traits::RepositoryError;

pub fn classify_database_error(error: &RepositoryError) -> String {
    let error_str = error.to_string();

    if error_str.contains("FOREIGN KEY constraint failed") {
        format!(
            "Database constraint error: Referenced entity does not exist - {}",
            error
        )
    } else if error_str.contains("UNIQUE constraint failed") {
        format!("Database constraint error: Duplicate entry - {}", error)
    } else if error_str.contains("NOT NULL constraint failed") {
        format!(
            "Database constraint error: Required field missing - {}",
            error
        )
    } else {
        format!("Database error: {}", error)
    }
}
