//! Roster query backing the Access Control page's department tree.


/// One user row for the access-control department tree.
#[derive(Debug, sqlx::FromRow)]
pub struct AccessTreeUserRow {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub department: String,
    pub is_active: bool,
}

// Why: Ordered by department, then display name.
//
// Anonymous accounts are excluded — they are never assignable principals.
