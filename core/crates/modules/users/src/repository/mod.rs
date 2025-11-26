use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_core_system::DbPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

use crate::models::users::{CreateUserRequest, UserResponse};

pub mod users {
    pub use super::UserRepository;
}

mod anonymous_user;
mod assign_roles;
mod create_user;
mod delete_user;
mod find_user;
mod list_users;
mod search_users;
mod update_user;

#[derive(Debug, Clone)]
pub struct UserRepository {
    db: Arc<dyn DatabaseProvider>,
    db_pool: DbPool,
}

impl RepositoryTrait for UserRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl UserRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db: db_pool.clone(),
            db_pool,
        }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse> {
        create_user::create_user(&*self.db, request).await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<UserResponse>> {
        find_user::find_by_name(&*self.db, name).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserResponse>> {
        find_user::find_by_email(&*self.db, email).await
    }

    pub async fn get_by_id(&self, user_id: &str) -> Result<Option<UserResponse>> {
        find_user::get_by_id(&*self.db, user_id).await
    }

    pub async fn find_first_admin(&self) -> Result<Option<UserResponse>> {
        find_user::find_first_admin(&*self.db).await
    }

    pub async fn find_first_user(&self) -> Result<Option<UserResponse>> {
        find_user::find_first_user(&*self.db).await
    }

    pub async fn find_by_role(&self, role: &str) -> Result<Option<UserResponse>> {
        find_user::find_by_role(&*self.db, role).await
    }

    pub async fn list_users(&self, filter: Option<&str>) -> Result<Vec<UserResponse>> {
        list_users::list_users(&*self.db, filter).await
    }

    pub async fn get_user_count(&self) -> Result<i64> {
        list_users::get_user_count(&*self.db).await
    }

    pub async fn search_users(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<UserResponse>> {
        search_users::search_users(&*self.db, query, limit, offset).await
    }

    pub async fn update_user(
        &self,
        name: &str,
        email: Option<&str>,
        full_name: Option<&str>,
        status: Option<&str>,
    ) -> Result<bool> {
        update_user::update_user(&*self.db, name, email, full_name, status).await
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<bool> {
        delete_user::delete_user(&*self.db, user_id).await
    }

    pub async fn assign_roles(&self, name: &str, roles: &[String]) -> Result<bool> {
        assign_roles::assign_roles(&*self.db, name, roles).await
    }

    pub async fn update_user_fields(
        &self,
        user_uuid: &str,
        request: &crate::models::users::UpdateUserRequest,
    ) -> Result<u64> {
        let query_enum = match (
            request.email.as_deref(),
            request.full_name.as_deref(),
            request.status.as_deref(),
        ) {
            (Some(_), Some(_), Some(_)) => DatabaseQueryEnum::UpdateUserAllFields,
            (Some(_), Some(_), None) => DatabaseQueryEnum::UpdateUserEmailFullName,
            (Some(_), None, Some(_)) => DatabaseQueryEnum::UpdateUserEmailStatus,
            (None, Some(_), Some(_)) => DatabaseQueryEnum::UpdateUserFullNameStatus,
            (Some(_), None, None) => DatabaseQueryEnum::UpdateUserEmail,
            (None, Some(_), None) => DatabaseQueryEnum::UpdateUserFullName,
            (None, None, Some(_)) => DatabaseQueryEnum::UpdateUserStatus,
            (None, None, None) => return Ok(0),
        };

        let query = query_enum.get(self.db.as_ref());

        let rows_affected = match (
            request.email.as_deref(),
            request.full_name.as_deref(),
            request.status.as_deref(),
        ) {
            (Some(e), Some(f), Some(s)) => {
                self.db.execute(&query, &[&e, &f, &s, &user_uuid]).await?
            },
            (Some(e), Some(f), None) => self.db.execute(&query, &[&e, &f, &user_uuid]).await?,
            (Some(e), None, Some(s)) => self.db.execute(&query, &[&e, &s, &user_uuid]).await?,
            (None, Some(f), Some(s)) => self.db.execute(&query, &[&f, &s, &user_uuid]).await?,
            (Some(e), None, None) => self.db.execute(&query, &[&e, &user_uuid]).await?,
            (None, Some(f), None) => self.db.execute(&query, &[&f, &user_uuid]).await?,
            (None, None, Some(s)) => self.db.execute(&query, &[&s, &user_uuid]).await?,
            (None, None, None) => return Ok(0),
        };

        Ok(rows_affected)
    }

    pub async fn create_anonymous_user(&self, user_id: &str) -> Result<()> {
        anonymous_user::create_anonymous_user(&*self.db, user_id).await
    }

    pub async fn delete_anonymous_user(&self, user_id: &str) -> Result<u64> {
        anonymous_user::delete_anonymous_user(&*self.db, user_id).await
    }

    pub async fn is_temporary_anonymous(&self, user_id: &str) -> Result<bool> {
        anonymous_user::is_temporary_anonymous(&*self.db, user_id).await
    }

    pub async fn cleanup_old_anonymous_users(&self) -> Result<u64> {
        anonymous_user::cleanup_old_anonymous_users(&*self.db).await
    }
}
