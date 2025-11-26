use anyhow::Result;
use systemprompt_core_users::models::users::api::CreateUserRequest;
use systemprompt_core_users::repository::UserRepository;

#[derive(Debug)]
pub struct UserCreationService {
    pub user_repo: UserRepository,
}

impl UserCreationService {
    pub const fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn find_or_create_user_with_webauthn_registration(
        &self,
        username: &str,
        email: &str,
        full_name: Option<&str>,
        roles: Option<Vec<String>>,
    ) -> Result<String> {
        if let Some(existing_user) = self.user_repo.find_by_email(email).await? {
            return Ok(existing_user.uuid);
        }

        let roles = roles.unwrap_or_else(|| vec!["user".to_string()]);

        let create_request = CreateUserRequest {
            name: username.to_string(),
            email: email.to_string(),
            full_name: full_name.map(String::from),
        };

        let user = self.user_repo.create_user(create_request).await?;

        self.user_repo.assign_roles(&user.name, &roles).await?;

        Ok(user.uuid)
    }

    pub async fn create_user_with_webauthn_registration(
        &self,
        username: &str,
        email: &str,
        full_name: Option<&str>,
    ) -> Result<String> {
        if self.user_repo.find_by_email(email).await?.is_some() {
            return Err(anyhow::anyhow!("email_already_registered"));
        }

        if self.user_repo.find_by_name(username).await?.is_some() {
            return Err(anyhow::anyhow!("username_already_taken"));
        }

        self.find_or_create_user_with_webauthn_registration(username, email, full_name, None)
            .await
    }
}
