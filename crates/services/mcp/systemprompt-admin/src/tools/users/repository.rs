use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use super::models::User;

pub struct UsersRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl UsersRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn list_users(&self, user_id: Option<&str>) -> Result<Vec<User>> {
        let query = DatabaseQueryEnum::ListUsers.get(self.db.as_ref());
        let rows = self.db.fetch_all(&query, &[&user_id]).await?;
        rows.iter().map(|r| User::from_json_row(r)).collect()
    }
}
