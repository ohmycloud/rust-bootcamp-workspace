use crate::models::ChatUser;
use crate::{AppError, AppState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, ToSchema)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_chat(&self, input: CreateChat, ws_id: i64) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }
        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        // verify if all member exist
        let users = ChatUser::fetch_by_ids(&input.members, &self.pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exists".to_string(),
            ));
        }
        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
        INSERT INTO chats(ws_id, name, type, members)
        VALUES ($1, $2, $3, $4)
        RETURNING id, ws_id, name, type, members, created_at
        "#,
        )
        .bind(ws_id)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_all(&self, ws_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
        SELECT id, ws_id, name, type, members, created_at
        FROM chats
        WHERE ws_id = $1"#,
        )
        .bind(ws_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn fetch_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
        SELECT id, ws_id, name, type, members, created_at
        FROM chats
        WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn is_chat_member(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 =ANY(members)
            "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(is_member.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state.create_chat(input, 1).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2, 3], false);
        let chat = state.create_chat(input, 1).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::Group);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_chat_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.fetch_by_id(1).await?.unwrap();

        assert_eq!(chat.id, 1);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_chat_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_all(1).await?;

        assert_eq!(chats.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<(), AppError> {
        let (_, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(-1, 1).await?;
        Ok(())
    }
}
