use crate::AppError;
use crate::models::workspace::Workspace;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use jwt_simple::prelude::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use std::mem;
use tracing::instrument;

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
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

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    #[instrument(name = "Creating a new user", skip(user, pool))]
    pub async fn create(user: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        // check if email exists
        let find_user = Self::find_by_email(&user.email, pool).await?;
        if find_user.is_some() {
            return Err(AppError::EmailAlreadyExists(user.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match Workspace::find_by_name(&user.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&user.workspace, 0, pool).await?,
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
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id, pool).await?;
        }

        Ok(user)
    }

    pub async fn verify(signin_user: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&signin_user.email)
        .fetch_optional(pool)
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
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        User {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: Utc::now(),
        }
    }

    pub async fn add_to_workspace(&self, ws_id: i64, pool: &PgPool) -> Result<User, AppError> {
        let user = sqlx::query_as(
            r#"
            UPDATE users
            SET ws_id = $1
            WHERE id = $2 and ws_id = 0
            RETURNING id, ws_id, fullname, email, created_at"#,
        )
        .bind(ws_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(user)
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

#[allow(dead_code)]
impl ChatUser {
    pub async fn fetch_by_ids(ids: &[i64], pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE id = ANY($1)
        "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn fetch_all(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1"#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::get_test_pool;
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
        let (_tdb, pool) = get_test_pool(None).await;
        let create_user = CreateUser::new("Alice Chen", "acme", "tchen@acme.org", "hunter42");
        let user = User::create(&create_user, &pool).await?;

        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(&create_user.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);

        let sign_user = SigninUser::new(&create_user.email, &create_user.password);
        let user = User::verify(&sign_user, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let email = "ohmycloudy@uk";
        let name = "ohmycloudy";
        let password = "hunter42";
        let created_user = CreateUser::new(name, "none", email, password);
        let user = User::create(&created_user, &pool).await?;

        assert_eq!(user.email, created_user.email);
        assert_eq!(user.fullname, created_user.fullname);
        assert_eq!(user.email, created_user.email);

        // Find a user
        let user = User::find_by_email(&created_user.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, created_user.fullname);
        assert_eq!(user.email, created_user.email);

        let signin_user = SigninUser::new(email, password);
        let user = User::verify(&signin_user, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
