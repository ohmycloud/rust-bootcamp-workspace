use crate::AppState;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chat_core::{AppError, User};
use serde::{Deserialize, Serialize};
use std::mem;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt: SaltString = SaltString::generate(&mut OsRng);

    // Argon2 with default params
    let argon2 = Argon2::default();

    // Hash password to PHC string
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    // Verify password
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

impl AppState {
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(name = "Creating a new user", skip(user))]
    pub async fn create_user(&self, user: &CreateUser) -> Result<User, AppError> {
        // check if email exists
        let find_user = self.find_user_by_email(&user.email).await?;
        if find_user.is_some() {
            return Err(AppError::EmailAlreadyExists(user.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match self.find_by_name(&user.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&user.workspace, 0).await?,
        };

        let password_hash = hash_password(&user.password)?;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, fullname, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, password_hash, created_at"#,
        )
        .bind(ws.id)
        .bind(&user.fullname)
        .bind(&user.email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            self.update_workspace_owner(user.id, ws.id).await?;
        }

        Ok(user)
    }

    pub async fn verify(&self, signin_user: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&signin_user.email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&signin_user.password, &password_hash.unwrap_or_default())?;
                if is_valid { Ok(Some(user)) } else { Ok(None) }
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, ws: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            workspace: ws.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunter42";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);

        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_user = CreateUser::new("Alice Chen", "acme", "tchen@acme.org", "hunter42");
        let user = state.create_user(&create_user).await?;

        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&create_user.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);

        let sign_user = SigninUser::new(&create_user.email, &create_user.password);
        let user = state.verify(&sign_user).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "ohmycloudy@uk";
        let name = "ohmycloudy";
        let password = "hunter42";
        let created_user = CreateUser::new(name, "none", email, password);
        let user = state.create_user(&created_user).await?;

        assert_eq!(user.email, created_user.email);
        assert_eq!(user.fullname, created_user.fullname);
        assert_eq!(user.email, created_user.email);

        // Find a user
        let user = state.find_user_by_email(&created_user.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, created_user.fullname);
        assert_eq!(user.email, created_user.email);

        let signin_user = SigninUser::new(email, password);
        let user = state.verify(&signin_user).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());

        let user = user.unwrap();
        assert_eq!(user.id, 1);

        Ok(())
    }
}
