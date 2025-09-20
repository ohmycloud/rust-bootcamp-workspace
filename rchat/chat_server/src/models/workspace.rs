use crate::models::ChatUser;
use crate::{AppError, AppState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::ToSchema;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

impl AppState {
    pub async fn create_workspace(&self, name: &str, owner_id: i64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
          INSERT INTO workspaces (name, owner_id)
          VALUES ($1, $2)
          RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn fetch_all_chat_users(&self, id: i64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1
        "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }
}

impl Workspace {
    pub async fn update_owner(&self, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 AND (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at"#,
        )
        .bind(owner_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateUser;
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_create_should_works() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;

        let input = CreateUser::new("Tom Yang", &ws.name, "toma@acme.org", "Hunter42");
        let user = state.create_user(&input).await?;

        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);
        assert_eq!(ws.owner_id, 0);

        let ws = ws.update_owner(user.id, &state.pool).await?;
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_name_should_works() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_by_name("acme").await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "acme");

        Ok(())
    }

    #[tokio::test]
    async fn workspace_fetch_all_chat_users_should_works() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let users = state.fetch_all_chat_users(1).await?;
        assert_eq!(users.len(), 5);

        Ok(())
    }
}
