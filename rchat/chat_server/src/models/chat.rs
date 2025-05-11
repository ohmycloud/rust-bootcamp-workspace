use crate::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: String,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq, sqlx::Type)]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

impl Chat {
    pub async fn create(input: CreateChat, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let chat = sqlx::query_as(
            r#"
        INSERT INTO chats(ws_id, name, type, members)
        VALUES ($1, $2, $3, $4)
        RETURING id, ws_id, name, r#type, members, created_at
        "#,
        )
        .bind(ws_id)
        .bind(input.name)
        .bind(ChatType::Group)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
}
