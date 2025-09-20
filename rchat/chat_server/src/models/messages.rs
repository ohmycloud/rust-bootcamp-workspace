use crate::models::ChatFile;
use crate::{AppError, AppState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListMessages {
    pub chat_id: i64,
    pub last_id: Option<i64>,
    pub limit: i64,
}

impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        // verify content - not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content cannot be empty".to_owned(),
            ));
        }
        // verify files
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} does not exist",
                    s
                )));
            }
        }

        // verify if user_id is a member of chat_id
        if !self.is_chat_member(chat_id, user_id).await? {
            return Err(AppError::CreateMessageError(format!(
                "User {user_id} are not a member of chat {chat_id}"
            )));
        }

        // create message
        let message: Message = sqlx::query_as(
            r#"
        INSERT INTO messages(chat_id, sender_id, content,files)
        VALUES ($1, $2, $3, $4)
        RETURNING id, chat_id, sender_id, content, files, created_at"#,
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn list_messages(&self, input: ListMessages) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX);
        let messages: Vec<Message> = sqlx::query_as(
            r#"
                 SELECT id, chat_id, sender_id, content, created_at
                 FROM messages
                 WHERE chat_id = $1
                 AND id < $2
                 ORDER BY id DESC
                 LIMIT $3"#,
        )
        .bind(input.chat_id)
        .bind(last_id)
        .bind(input.limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message failed");
        assert_eq!(message.content, "hello");

        // invalid files should fail
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec!["1".to_string()],
        };
        let err = state.create_message(input, 1, 1).await.unwrap_err();
        assert_eq!(err.to_string(), "Invalid chat file path: 1");

        // valid files should work
        let url = upload_dummy_file(&state)?;
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![url],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message failed");
        assert_eq!(message.content, "hello");

        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exists"))?;
        std::fs::write(&path, b"hello world")?;

        Ok(file.url())
    }

    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
        assert!(is_member);

        // user 6 doesn't exit
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        // chat 10 doesn't exit
        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        // user 4 is not a member of chat 2
        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);

        Ok(())
    }

    #[tokio::test]
    async fn list_messages_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ListMessages {
            chat_id: 1,
            last_id: None,
            limit: 6,
        };

        let messages: Vec<Message> = state.list_messages(input).await?;
        assert_eq!(messages.len(), 6);

        let last_id = messages.last().expect("last message should exists").id;
        let input = ListMessages {
            chat_id: 1,
            last_id: Some(last_id),
            limit: 6,
        };
        let messages: Vec<Message> = state.list_messages(input).await?;
        assert_eq!(messages.len(), 4);

        Ok(())
    }
}
