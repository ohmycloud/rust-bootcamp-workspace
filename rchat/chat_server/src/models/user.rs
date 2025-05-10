use crate::AppError;
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
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        let user = sqlx::query_as("SELECT id, fullname, created_at FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    #[instrument(name = "Creating a new user", skip(user, pool))]
    pub async fn create(user: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = hash_password(&user.password)?;
        let user = sqlx::query_as(
            r#"
        INSERT INTO users (fullname, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, fullname, email, password_hash, created_at"#,
        )
        .bind(&user.fullname)
        .bind(&user.email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(signin_user: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
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
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
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

impl ChatUser {
    pub async fn fetch_all(user: &User, pool: &PgPool) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunter42";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(&password, &password_hash)?);

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:possword@localhost:5432/rchat".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let email = "ohmycloudy@uk";
        let name = "ohmycloudy";
        let password = "hunter42";
        let created_user = CreateUser::new(&name, &email, &password);
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

        let signin_user = SigninUser::new(&email, &password);
        let user = User::verify(&signin_user, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
