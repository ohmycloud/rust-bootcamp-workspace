use crate::AppError;
use crate::models::ChatUser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

impl Workspace {
    pub async fn create(name: &str, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
          INSERT INTO workspaces (name, owner_id)
          VALUES ($1, $2)
          RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1 WHERE id = $2 AND (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at"#,
        )
        .bind(owner_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn fetch_all_chat_users(id: i64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1
        "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::User;
    use crate::models::CreateUser;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[tokio::test]
    async fn workspace_create_should_works() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:password@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let input = CreateUser::new("Alice", "none", "alice@acme.org", "Hunter42");
        let user = User::create(&input, &pool).await?;
        let ws = Workspace::create("test", 0, &pool).await?;
        let user = user.add_to_workspace(ws.id, &pool).await?;
        assert_eq!(user.ws_id, ws.id);
        assert_eq!(ws.name, "test");
        assert_eq!(ws.owner_id, 0);

        let ws = ws.update_owner(user.id, &pool).await?;
        assert_eq!(ws.owner_id, 1);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_name_should_works() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:password@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let _ws = Workspace::create("test", 0, &pool).await?;
        let ws = Workspace::find_by_name("test", &pool).await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_fetch_all_chat_users_should_works() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:password@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create("test", 0, &pool).await?;
        let input = CreateUser::new("Alice", &ws.name, "alice@acme.org", "Hunter42");
        let user1 = User::create(&input, &pool).await?;

        let input = CreateUser::new("Tom", &ws.name, "tom@acme.org", "Hunter42");
        let user2 = User::create(&input, &pool).await?;

        let users = Workspace::fetch_all_chat_users(ws.id, &pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);

        Ok(())
    }
}
